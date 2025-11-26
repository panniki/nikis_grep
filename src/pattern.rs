use crate::errors::PatternError;
use std::{iter::Peekable, slice::Iter, str::Chars};

#[derive(Debug, PartialEq, Eq)]
enum Quantifier {
    OneOrMore(Atom), //+
    ZeroOrOne(Atom), // ?
    Exact(Atom),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Atom {
    FromStart,     // ^
    ToEnd,         // $
    Digit,         // \d
    W,             // \w
    Literal(char), // abcdeAbcdzzz231237
    Chars(Vec<Atom>, bool),
}

pub struct Pattern {
    body: Vec<Quantifier>,
}

impl Pattern {
    pub fn new(input: &str) -> Result<Self, PatternError> {
        let mut body = vec![];
        let mut input_chars = input.chars().peekable();

        while let Some(curr_char) = input_chars.next() {
            match curr_char {
                // class
                '\\' => {
                    if let Some(next_char) = input_chars.next() {
                        let atom = Self::parse_atom(&next_char);
                        body.push(Self::quantify(&mut input_chars, atom))
                    } else {
                        return Err(PatternError::NoClassFound);
                    }
                }
                // char class.
                '[' => {
                    let mut char_class: Vec<Atom> = vec![];
                    let mut found_closing = false;
                    let mut is_positive = true;

                    while let Some(c) = input_chars.next() {
                        match c {
                            ']' => {
                                found_closing = true;
                                break;
                            }
                            '^' => {
                                is_positive = false;
                            }
                            '\\' => {
                                let class: Atom;
                                if let Some(next_c) = input_chars.next() {
                                    class = Self::parse_atom(&next_c);
                                } else {
                                    class = Self::parse_atom(&c);
                                }
                                char_class.push(class);
                            }
                            _ => char_class.push(Atom::Literal(c)),
                        }
                    }

                    if !found_closing {
                        return Err(PatternError::InvalidCharClass);
                    }
                    let atom = Atom::Chars(char_class, is_positive);
                    body.push(Self::quantify(&mut input_chars, atom))
                }
                '^' => body.push(Quantifier::Exact(Atom::FromStart)),
                '$' => body.push(Quantifier::Exact(Atom::ToEnd)),
                // literal
                c => {
                    let atom = Atom::Literal(c);
                    body.push(Self::quantify(&mut input_chars, atom))
                }
            }
        }

        Ok(Self { body })
    }

    fn quantify(chars: &mut Peekable<Chars<'_>>, atom: Atom) -> Quantifier {
        if let Some(&peek) = chars.peek() {
            chars.next_if(|&c| matches!(c, '+' | '*' | '?'));

            match peek {
                '+' => Quantifier::OneOrMore(atom),
                '*' => unimplemented!("Zero or more."),
                '?' => Quantifier::ZeroOrOne(atom),
                _ => Quantifier::Exact(atom),
            }
        } else {
            Quantifier::Exact(atom)
        }
    }

    fn parse_atom(c: &char) -> Atom {
        match c {
            'd' => Atom::Digit,
            'w' => Atom::W,
            '\\' => Atom::Literal('\\'),
            x => unimplemented!("not supported yet: {x}"),
        }
    }

    pub fn is_match(&self, input: &str) -> bool {
        if input.is_empty() || self.body.is_empty() {
            return false;
        }
        let mut found = false;
        let mut chars_iter = input.chars().peekable();
        let mut i = 0;
        let mut expr_iter = self.body.iter().peekable();
        let mut maybe_matcher = expr_iter.next();

        let mut allow_unmatch = true;
        let mut maybe_inp_char = chars_iter.next();

        while let Some(inp_char) = maybe_inp_char {
            if let Some(matcher) = maybe_matcher {
                if !allow_unmatch && !found {
                    break;
                }

                found = match matcher {
                    Quantifier::Exact(atom) => match atom {
                        Atom::Digit | Atom::W | Atom::Literal(_) | Atom::Chars(_, _) => {
                            Self::match_atom(&inp_char, atom)
                        }
                        Atom::FromStart => {
                            if i == 0 {
                                maybe_matcher = expr_iter.next();
                                allow_unmatch = false;
                                found = true;
                                continue;
                            } else {
                                break;
                            }
                        }
                        // if we match it here it means that that input_str longer and does not
                        // match the regex.
                        Atom::ToEnd => false,
                    },
                    Quantifier::OneOrMore(atom) => {
                        Self::count(&inp_char, atom, &mut chars_iter, &mut expr_iter) >= 1
                    }
                    Quantifier::ZeroOrOne(atom) => unimplemented!(),
                };
                if found {
                    maybe_matcher = expr_iter.next();
                }
            } else {
                break;
            }
            maybe_inp_char = chars_iter.next();
            i += 1;
        }
        let is_final = if let Some(Quantifier::Exact(Atom::ToEnd)) = maybe_matcher {
            true
        } else {
            maybe_matcher.is_none()
        };
        found && is_final
    }

    fn count(
        curr_char: &char,
        curr_atom: &Atom,
        chars: &mut Peekable<Chars<'_>>,
        expr: &mut Peekable<Iter<Quantifier>>,
    ) -> usize {
        if !Self::match_atom(curr_char, curr_atom) {
            return 0;
        }
        let mut counter = 1;

        while let Some(peek) = chars.peek() {
            if Self::match_atom(peek, curr_atom) {
                counter += 1;
                chars.next();
            } else {
                break;
            }
        }

        if let Some(Quantifier::Exact(next_atom)) = expr.peek() {
            // handle "ca+at"
            if next_atom == curr_atom {
                expr.next();
            }
        }
        counter
    }

    pub fn match_atom(in_char: &char, atom: &Atom) -> bool {
        match atom {
            Atom::Digit => in_char.is_ascii_digit(),
            Atom::Literal(literal) => literal == in_char,
            Atom::W => in_char.is_ascii_digit() || in_char.is_ascii_alphabetic() || in_char == &'_',
            Atom::Chars(cc, pos) => {
                let mtch = cc.iter().any(|c| Self::match_atom(in_char, c));

                if *pos {
                    mtch
                } else {
                    !mtch
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_digit() -> Result<(), PatternError> {
        let input = r"\d";
        let ptrn = Pattern::new(input)?;
        // assert_eq!(ptrn.body.anchor, Anchor::Free);
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn parse_iteral() -> Result<(), PatternError> {
        let input = r"abc123\d";
        let ptrn = Pattern::new(input)?;
        // assert_eq!(ptrn.body.anchor, Anchor::Free);
        assert_eq!(ptrn.body.len(), 7);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('a'))
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal('b'))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('c'))
        );
        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::Exact(Atom::Literal('1'))
        );
        assert_eq!(
            ptrn.body.get(4).unwrap(),
            &Quantifier::Exact(Atom::Literal('2'))
        );
        assert_eq!(
            ptrn.body.get(5).unwrap(),
            &Quantifier::Exact(Atom::Literal('3'))
        );
        assert_eq!(ptrn.body.get(6).unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn match_digit() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\d")?;
        assert!(ptrn.is_match("3"));
        assert!(ptrn.is_match("12312412512"));
        assert!(!ptrn.is_match("nope"));

        Ok(())
    }

    #[test]
    fn parse_word_char() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\d\w\w\d")?;
        assert_eq!(ptrn.body.len(), 4);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(ptrn.body.get(1).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.body.get(3).unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn match_word_char() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\w")?;
        assert!(ptrn.is_match(r"148"));
        assert!(ptrn.is_match(r"ORAnge"));
        assert!(ptrn.is_match(r"-+÷_-+="));

        Ok(())
    }

    #[test]
    fn parse_basic_char_atom() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"[abcde\d\w]")?;
        let char_atom = Atom::Chars(
            vec![
                Atom::Literal('a'),
                Atom::Literal('b'),
                Atom::Literal('c'),
                Atom::Literal('d'),
                Atom::Literal('e'),
                Atom::Digit,
                Atom::W,
            ],
            true,
        );
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(char_atom));

        Ok(())
    }

    #[test]
    fn parse_adv_char_atom() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"[abcde\d\w]\d\w322")?;
        let char_atom = Atom::Chars(
            vec![
                Atom::Literal('a'),
                Atom::Literal('b'),
                Atom::Literal('c'),
                Atom::Literal('d'),
                Atom::Literal('e'),
                Atom::Digit,
                Atom::W,
            ],
            true,
        );
        assert_eq!(ptrn.body.len(), 6);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(char_atom));
        assert_eq!(ptrn.body.get(1).unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::Exact(Atom::Literal('3'))
        );
        assert_eq!(
            ptrn.body.get(4).unwrap(),
            &Quantifier::Exact(Atom::Literal('2'))
        );
        assert_eq!(
            ptrn.body.get(5).unwrap(),
            &Quantifier::Exact(Atom::Literal('2'))
        );

        Ok(())
    }

    #[test]
    fn invalid_char_atom() {
        assert!(Pattern::new("[abcd").is_err())
    }

    #[test]
    fn match_basic_char_atom() -> Result<(), PatternError> {
        let ptrn = Pattern::new("[raspberry]")?;
        assert!(ptrn.is_match("p"));
        assert!(!ptrn.is_match(""));

        let ptrn = Pattern::new("[grape]")?;
        assert!(ptrn.is_match("gbc"));

        let ptrn = Pattern::new("[acdfghijk]")?;
        assert!(!ptrn.is_match("blueberry"));
        Ok(())
    }

    #[test]
    fn parse_negative_char_atom() -> Result<(), PatternError> {
        let ptrn = Pattern::new("[^abcd]")?;
        let char_atom = Atom::Chars(
            vec![
                Atom::Literal('a'),
                Atom::Literal('b'),
                Atom::Literal('c'),
                Atom::Literal('d'),
            ],
            false,
        );
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(char_atom));

        Ok(())
    }

    #[test]
    fn match_basic_ne_char_atom() -> Result<(), PatternError> {
        let ptrn = Pattern::new("[^abc]")?;
        assert!(ptrn.is_match("cat")); // cuz t not in the set.
        assert!(!ptrn.is_match("cab"));

        Ok(())
    }

    #[test]
    fn match_sequence() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\d apple")?;
        assert_eq!(ptrn.body.len(), 7);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal(' '))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('a'))
        );
        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::Exact(Atom::Literal('p'))
        );
        assert_eq!(
            ptrn.body.get(4).unwrap(),
            &Quantifier::Exact(Atom::Literal('p'))
        );
        assert_eq!(
            ptrn.body.get(5).unwrap(),
            &Quantifier::Exact(Atom::Literal('l'))
        );
        assert_eq!(
            ptrn.body.get(6).unwrap(),
            &Quantifier::Exact(Atom::Literal('e'))
        );

        assert!(ptrn.is_match("1 apple"));
        assert!(!ptrn.is_match("1 orange"));

        Ok(())
    }

    #[test]
    fn not_match_sequence_when_its_not_completed() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\d \w\w\ws")?;
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal(' '))
        );
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.body.get(3).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.body.get(4).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(
            ptrn.body.get(5).unwrap(),
            &Quantifier::Exact(Atom::Literal('s'))
        );

        assert!(!ptrn.is_match("sally has 1 dog"));
        Ok(())
    }

    #[test]
    fn parse_from_start_anchor() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"^test")?;
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::FromStart)
        );

        Ok(())
    }

    #[test]
    fn match_from_start() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"^log")?;
        assert!(ptrn.is_match("log"));

        let ptrn = Pattern::new(r"^log")?;
        assert!(ptrn.is_match("logs"));

        let ptrn = Pattern::new(r"^logs")?;
        assert!(!ptrn.is_match("slog"));

        let ptrn = Pattern::new(r"^\d\d\d")?;
        assert!(ptrn.is_match("123abc"));
        assert!(!ptrn.is_match("abc123"));

        Ok(())
    }

    #[test]
    fn parse_to_end_anchor() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"test$")?;
        assert_eq!(ptrn.body.last().unwrap(), &Quantifier::Exact(Atom::ToEnd));

        Ok(())
    }

    #[test]
    fn parse_from_start_to_end() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"^test$")?;
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::FromStart)
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal('t'))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('e'))
        );
        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::Exact(Atom::Literal('s'))
        );
        assert_eq!(
            ptrn.body.get(4).unwrap(),
            &Quantifier::Exact(Atom::Literal('t'))
        );
        assert_eq!(ptrn.body.last().unwrap(), &Quantifier::Exact(Atom::ToEnd));

        Ok(())
    }

    #[test]
    fn match_to_end() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"dog$")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("hotdog"));
        assert!(!ptrn.is_match("dogs"));

        let ptrn = Pattern::new(r"\d\d\d$")?;
        assert!(ptrn.is_match("abc123"));

        let ptrn = Pattern::new(r"\w\w\w$")?;
        assert!(!ptrn.is_match("abc123@"));

        let ptrn = Pattern::new(r"\w\w\w$")?;
        assert!(!ptrn.is_match("abc123cde"));

        Ok(())
    }

    #[test]
    fn parse_one_or_more_qntf() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"\d+")?;
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::OneOrMore(Atom::Digit)
        );

        let ptrn = Pattern::new(r"[abc1\d\w]+de")?;
        let char_atom = Atom::Chars(
            vec![
                Atom::Literal('a'),
                Atom::Literal('b'),
                Atom::Literal('c'),
                Atom::Literal('1'),
                Atom::Digit,
                Atom::W,
            ],
            true,
        );
        assert_eq!(ptrn.body.len(), 3);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::OneOrMore(char_atom)
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('e'))
        );

        let ptrn = Pattern::new(r"ca+ts")?;
        assert_eq!(ptrn.body.len(), 4);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('c'))
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::OneOrMore(Atom::Literal('a'))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('t'))
        );

        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::Exact(Atom::Literal('s'))
        );

        Ok(())
    }

    // #[test]
    // fn count_occurance() -> Result<(), PatternError> {
    //     let mut test = "ttest".chars().peekable();
    //     let first_one = test.next().unwrap();
    //     let res = Pattern::count(&first_one, &mut test, &Class::Literal('t'));
    //     assert_eq!(res, 2);
    //
    //     let mut test = "Tttest".chars().peekable();
    //     let first_one = test.next().unwrap();
    //     let res = Pattern::count(&first_one, &mut test, &Class::Literal('t'));
    //     assert_eq!(res, 0);
    //
    //     let mut test = "123456abasdfs".chars().peekable();
    //     let first_one = test.next().unwrap();
    //     let res = Pattern::count(&first_one, &mut test, &Class::Digit);
    //     assert_eq!(res, 6);
    //
    //     let mut test = "123456abasdfs".chars().peekable();
    //     let first_one = test.next().unwrap();
    //     let res = Pattern::count(&first_one, &mut test, &Class::W);
    //     assert_eq!(res, 13);
    //
    //     Ok(())
    // }

    #[test]
    fn match_char_atom() -> Result<(), PatternError> {
        let char_atom = Atom::Chars(
            vec![
                Atom::Literal('g'),
                Atom::Literal('r'),
                Atom::Literal('a'),
                Atom::Literal('p'),
                Atom::Literal('e'),
            ],
            true,
        );
        assert!(Pattern::match_atom(&'g', &char_atom));
        assert!(!Pattern::match_atom(&'z', &char_atom));
        assert!(Pattern::match_atom(&'p', &char_atom));
        Ok(())
    }

    #[test]
    fn match_one_or_more_qntf() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"a+")?;
        assert!(!ptrn.is_match("dog"));
        assert!(ptrn.is_match("SaaS"));
        assert!(ptrn.is_match("SaS"));

        let ptrn = Pattern::new(r"ca+ts")?;
        assert!(ptrn.is_match("cats"));
        assert!(ptrn.is_match("caats"));
        assert!(!ptrn.is_match("cts"));

        let ptrn = Pattern::new(r"ca+ts")?;
        assert!(ptrn.is_match("caaats"));

        let ptrn = Pattern::new(r"^abc_\d+_xyz$")?;
        assert!(ptrn.is_match("abc_123_xyz"));

        Ok(())
    }

    #[test]
    fn parse_once_or_not() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"dogs?")?;
        assert_eq!(ptrn.body.len(), 4);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );

        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::Exact(Atom::Literal('o'))
        );
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('g'))
        );
        assert_eq!(
            ptrn.body.get(3).unwrap(),
            &Quantifier::ZeroOrOne(Atom::Literal('s'))
        );

        let ptrn = Pattern::new(r"[abc]?\d?\w?")?;
        let char_atom = Atom::Chars(
            vec![Atom::Literal('a'), Atom::Literal('b'), Atom::Literal('c')],
            true,
        );
        assert_eq!(ptrn.body.len(), 3);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::ZeroOrOne(char_atom)
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::ZeroOrOne(Atom::Digit)
        );
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::ZeroOrOne(Atom::W));

        Ok(())
    }
}
