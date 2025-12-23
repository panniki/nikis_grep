use crate::errors::PatternError;
use std::{iter::Peekable, slice::Iter, str::Chars};

#[derive(Debug, PartialEq, Eq)]
enum Quantifier {
    OneOrMore(Atom), // +
    ZeroOrOne(Atom), // ?
    Exact(Atom),
}

impl Quantifier {
    fn get_atom(&self) -> &Atom {
        match self {
            Self::Exact(atom) | Self::ZeroOrOne(atom) | Self::OneOrMore(atom) => atom,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Atom {
    FromStart,                      // ^
    ToEnd,                          // $
    Digit,                          // \d
    W,                              // \w
    Literal(char),                  // abcdeAbcdzzz231237
    Chars(Vec<Atom>, bool),         // [foo322]
    Any,                            // .
    AltGroup(Vec<Vec<Quantifier>>), // (cat|dog)
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
                '(' => {
                    let mut group: Vec<Vec<Quantifier>> = vec![];

                    let mut found_closing = false;

                    while let Some(c) = input_chars.next() {
                        match c {
                            ')' => {
                                found_closing = true;
                                break;
                            }
                            '|' => continue,
                            cc => {
                                let prim = Self::parse_primitives(&mut input_chars, cc)?;
                                group.push(prim);
                            }
                        }
                    }

                    if !found_closing {
                        return Err(PatternError::InvalidGroup);
                    }

                    body.push(Self::quantify(&mut input_chars, Atom::AltGroup(group)))
                }
                // primetives
                cc => {
                    let mut prim = Self::parse_primitives(&mut input_chars, cc)?;
                    body.append(&mut prim);
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

    fn parse_primitives(
        input_chars: &mut Peekable<Chars<'_>>,
        cc: char,
    ) -> Result<Vec<Quantifier>, PatternError> {
        let mut body = vec![];
        let mut maybe_cc = Some(cc);

        while let Some(curr_char) = maybe_cc {
            match curr_char {
                '.' => {
                    let atom = Atom::Any;
                    body.push(Self::quantify(input_chars, atom))
                }
                // class
                '\\' => {
                    if let Some(next_char) = input_chars.next() {
                        let atom = Self::parse_atom(&next_char);
                        body.push(Self::quantify(input_chars, atom))
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
                    body.push(Self::quantify(input_chars, atom))
                }
                '^' => body.push(Quantifier::Exact(Atom::FromStart)),
                '$' => body.push(Quantifier::Exact(Atom::ToEnd)),
                // literal
                c => {
                    let atom = Atom::Literal(c);
                    body.push(Self::quantify(input_chars, atom))
                }
            }

            match input_chars.peek() {
                Some('|') | Some(')') | Some('(') => break,
                _ => {
                    maybe_cc = input_chars.next();
                }
            }
        }

        Ok(body)
    }

    fn parse_atom(c: &char) -> Atom {
        match c {
            'd' => Atom::Digit,
            'w' => Atom::W,
            '\\' => Atom::Literal('\\'),
            x => unimplemented!("not supported yet: {x}"),
        }
    }

    /// Returns the amount of consumed characters or None if no match was found.
    pub fn match_from(pattern: &[Quantifier], input: &str) -> Option<usize> {
        if pattern.is_empty() {
            return None;
        }

        let mut consumed = 0;
        let mut found = false;
        let mut allow_unmatch = true;
        let mut i = 0;
        let mut chars_iter = input.chars().peekable();
        let mut ptrn_iter = pattern.iter().peekable();
        let mut maybe_matcher = ptrn_iter.next();
        let mut maybe_inp_char = chars_iter.next();

        // Handle empty input & ?
        if maybe_inp_char.is_none() {
            if let Some(Quantifier::ZeroOrOne(_)) = maybe_matcher {
                return Some(consumed);
            } else {
                return None;
            }
        }

        while let Some(inp_char) = maybe_inp_char {
            if let Some(matcher) = maybe_matcher {
                if !allow_unmatch && !found {
                    break;
                }

                found = match matcher {
                    Quantifier::Exact(atom) => match atom {
                        Atom::Digit
                        | Atom::W
                        | Atom::Literal(_)
                        | Atom::Any
                        | Atom::Chars(_, _) => {
                            let mtch = Self::match_atom(&inp_char, atom);
                            if mtch {
                                consumed += 1;
                            }
                            mtch
                        }
                        Atom::FromStart => {
                            if i == 0 {
                                maybe_matcher = ptrn_iter.next();
                                allow_unmatch = false;
                                found = true;
                                continue;
                            } else {
                                break;
                            }
                        }
                        // if we match it here it means that input is longer and does not
                        // match the regex.
                        Atom::ToEnd => false,
                        Atom::AltGroup(_atom_list) => unimplemented!("(cat|dog)"),
                    },
                    Quantifier::OneOrMore(atom) => {
                        let count = Self::count(&inp_char, atom, &mut chars_iter, &mut ptrn_iter);
                        consumed += count;
                        count >= 1
                    }
                    Quantifier::ZeroOrOne(atom) => {
                        let count = Self::count(&inp_char, atom, &mut chars_iter, &mut ptrn_iter);
                        consumed += count;
                        let mtch = count <= 1;

                        // handles case when regex longer than input
                        // ex: /colou?r/ -> "color"
                        if mtch && chars_iter.peek().is_none() {
                            consumed += 1;
                            maybe_matcher = ptrn_iter.next();
                        }

                        mtch
                    }
                };
                if found {
                    allow_unmatch = false;
                    maybe_matcher = ptrn_iter.next();
                }
            } else {
                break;
            }
            maybe_inp_char = chars_iter.next();
            i += 1;
        }

        let is_final = if let Some(Quantifier::Exact(Atom::ToEnd)) = maybe_matcher {
            true
        } else if let Some(Quantifier::ZeroOrOne(_)) = maybe_matcher {
            true
        } else {
            maybe_matcher.is_none()
        };

        if found && is_final {
            Some(consumed)
        } else {
            None
        }
    }

    pub fn is_match(&self, input: &str) -> bool {
        Self::match_from(&self.body, input).is_some()
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
            let maybe_next_atom = expr.peek().map(|q| q.get_atom());
            match (Self::match_atom(peek, curr_atom), maybe_next_atom) {
                (true, Some(next_atom)) => {
                    if Self::match_atom(peek, next_atom) && next_atom != curr_atom {
                        break;
                    } else {
                        if next_atom == curr_atom {
                            expr.next();
                        }

                        counter += 1;
                        chars.next();
                    }
                }
                (true, None) => {
                    counter += 1;
                    chars.next();
                }
                _ => break,
            }
        }
        counter
    }

    fn match_atom(in_char: &char, atom: &Atom) -> bool {
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
            Atom::Any => in_char != &'\n',
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

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
    fn parse_digit() -> Result<(), PatternError> {
        let input = r"\d";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn parses_anychar() -> Result<(), PatternError> {
        let input = r"d.g";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.body.len(), 3);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );
        assert_eq!(ptrn.body.get(1).unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(
            ptrn.body.get(2).unwrap(),
            &Quantifier::Exact(Atom::Literal('g'))
        );

        let input = r"...";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.body.len(), 3);
        assert_eq!(ptrn.body.first().unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(ptrn.body.get(1).unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::Exact(Atom::Any));

        let input = r"d.?.+";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.body.len(), 3);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );
        assert_eq!(ptrn.body.get(1).unwrap(), &Quantifier::ZeroOrOne(Atom::Any));
        assert_eq!(ptrn.body.get(2).unwrap(), &Quantifier::OneOrMore(Atom::Any));
        Ok(())
    }

    #[test]
    fn parse_iteral() -> Result<(), PatternError> {
        let input = r"abc123\d";
        let ptrn = Pattern::new(input)?;
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
    fn parse_from_start_anchor() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"^test")?;
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::FromStart)
        );

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

    #[test]
    fn count_occurance() -> Result<(), PatternError> {
        let mut test = "ttest".chars().peekable();
        let first_one = test.next().unwrap();
        let res = Pattern::count(
            &first_one,
            &Atom::Literal('t'),
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 2);

        let mut test = "Tttest".chars().peekable();
        let first_one = test.next().unwrap();
        let res = Pattern::count(
            &first_one,
            &Atom::Literal('t'),
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 0);

        let mut test = "123456abasdfs".chars().peekable();
        let first_one = test.next().unwrap();
        let res = Pattern::count(
            &first_one,
            &Atom::Digit,
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 6);

        let mut test = "123456abasdfs".chars().peekable();
        let first_one = test.next().unwrap();
        let res = Pattern::count(
            &first_one,
            &Atom::W,
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 13);

        // Test from middle position
        let mut test = "abc333def".chars().peekable();
        test.next(); // skip 'a'
        test.next(); // skip 'b'
        test.next(); // skip 'c'
        let mid_char = test.next().unwrap();
        let res = Pattern::count(
            &mid_char,
            &Atom::Digit,
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 3);

        // Test from end position
        let mut test = "aaabbbccc".chars().peekable();
        for _ in 0..6 {
            test.next();
        } // skip to 'c'
        let end_char = test.next().unwrap();
        let res = Pattern::count(
            &end_char,
            &Atom::Literal('c'),
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 3);

        // Test non-match from middle
        let mut test = "aaa123bbb".chars().peekable();
        for _ in 0..3 {
            test.next();
        } // skip to '1'
        let mid_char = test.next().unwrap();
        let res = Pattern::count(
            &mid_char,
            &Atom::Literal('x'),
            &mut test,
            &mut vec![].iter().peekable(),
        );
        assert_eq!(res, 0);

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

    #[test]
    fn parse_alt_group() -> Result<(), PatternError> {
        let ptrn = Pattern::new(r"(c+at|dog?)([\dog]?|[\wod]+)?")?;
        assert_eq!(ptrn.body.len(), 2);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::AltGroup(vec![
                vec![
                    Quantifier::OneOrMore(Atom::Literal('c')),
                    Quantifier::Exact(Atom::Literal('a')),
                    Quantifier::Exact(Atom::Literal('t'))
                ],
                vec![
                    Quantifier::Exact(Atom::Literal('d')),
                    Quantifier::Exact(Atom::Literal('o')),
                    Quantifier::ZeroOrOne(Atom::Literal('g'))
                ]
            ]))
        );
        assert_eq!(
            ptrn.body.get(1).unwrap(),
            &Quantifier::ZeroOrOne(Atom::AltGroup(vec![
                vec![Quantifier::ZeroOrOne(Atom::Chars(
                    vec![Atom::Digit, Atom::Literal('o'), Atom::Literal('g')],
                    true
                ))],
                vec![Quantifier::OneOrMore(Atom::Chars(
                    vec![Atom::W, Atom::Literal('o'), Atom::Literal('d')],
                    true
                ))],
            ]))
        );
        let ptrn = Pattern::new(r"(cat|dog|\d\w)")?;
        assert_eq!(ptrn.body.len(), 1);
        assert_eq!(
            ptrn.body.first().unwrap(),
            &Quantifier::Exact(Atom::AltGroup(vec![
                vec![
                    Quantifier::Exact(Atom::Literal('c')),
                    Quantifier::Exact(Atom::Literal('a')),
                    Quantifier::Exact(Atom::Literal('t'))
                ],
                vec![
                    Quantifier::Exact(Atom::Literal('d')),
                    Quantifier::Exact(Atom::Literal('o')),
                    Quantifier::Exact(Atom::Literal('g'))
                ],
                vec![Quantifier::Exact(Atom::Digit), Quantifier::Exact(Atom::W),]
            ]))
        );

        Ok(())
    }

    #[test]
    fn match_atom() -> Result<(), PatternError> {
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
    fn match_from_basic_literals() -> Result<(), PatternError> {
        // Pattern "cat" on input "cat" → Some(3)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "cat"), Some(3));
        assert_eq!(Pattern::match_from(ptrn, "dog"), None);

        // Pattern "do" on input "dog" → Some(2)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('d')),
            Quantifier::Exact(Atom::Literal('o')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "dog"), Some(2));

        Ok(())
    }

    #[test]
    fn match_from_one_or_more_quantifier() -> Result<(), PatternError> {
        // Pattern "c+at" on input "ccat" → Some(4)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "ccat"), Some(4));
        assert_eq!(Pattern::match_from(ptrn, "cccccat"), Some(7));

        // Pattern "\d+" on input "12345abc" → Some(5)
        let ptrn = &[Quantifier::OneOrMore(Atom::Digit)];
        assert_eq!(Pattern::match_from(ptrn, "12345abc"), Some(5));

        Ok(())
    }

    #[test]
    fn match_from_zero_or_one_quantifier() -> Result<(), PatternError> {
        // Pattern "colou?r" on input "color" → Some(5)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('o')),
            Quantifier::Exact(Atom::Literal('l')),
            Quantifier::Exact(Atom::Literal('o')),
            Quantifier::ZeroOrOne(Atom::Literal('u')),
            Quantifier::Exact(Atom::Literal('r')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "color"), Some(5));
        assert_eq!(Pattern::match_from(ptrn, "colour"), Some(6));

        // Pattern "\d?" on input "foo" → Some(0)
        let ptrn = &[Quantifier::ZeroOrOne(Atom::Digit)];
        assert_eq!(Pattern::match_from(ptrn, "foo"), Some(0));

        Ok(())
    }

    #[test]
    fn match_from_multiple_quantifiers() -> Result<(), PatternError> {
        // Pattern "a+b+c" on input "aaabbbccc" → Some(9)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::OneOrMore(Atom::Literal('b')),
            Quantifier::OneOrMore(Atom::Literal('c')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "aaabbbccc"), Some(9));

        // Pattern "\d+\w+" on input "123abc" → Some(6)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Digit),
            Quantifier::OneOrMore(Atom::W),
        ];
        assert_eq!(Pattern::match_from(ptrn, "123abc"), Some(6));

        Ok(())
    }

    #[test]
    fn match_from_greedy_quantifiers() -> Result<(), PatternError> {
        // Pattern "a+a" on input "aaa" → Some(3)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('a')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "aaa"), Some(3));
        assert_eq!(Pattern::match_from(ptrn, "aa"), Some(2));
        assert_eq!(Pattern::match_from(ptrn, "a"), None);

        // TODO: fix this one.
        // Pattern ".*cat" on input "the cat" → Some(7)
        // let ptrn = &[
        //     Quantifier::ZeroOrOne(Atom::Any),
        //     Quantifier::Exact(Atom::Literal('c')),
        //     Quantifier::Exact(Atom::Literal('a')),
        //     Quantifier::Exact(Atom::Literal('t')),
        // ];
        // assert_eq!(Pattern::match_from(ptrn, "the cat"), Some(7));

        Ok(())
    }

    #[test]
    fn match_from_anchors() -> Result<(), PatternError> {
        // Pattern "^cat" tests
        let ptrn = &[
            Quantifier::Exact(Atom::FromStart),
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(Pattern::match_from(ptrn, "cat"), Some(3));
        assert_eq!(Pattern::match_from(ptrn, "dog cat"), None);

        // Pattern "cat$" tests
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
            Quantifier::Exact(Atom::ToEnd),
        ];
        assert_eq!(Pattern::match_from(ptrn, "cat"), Some(3));
        assert_eq!(Pattern::match_from(ptrn, "dog cat"), Some(3));

        Ok(())
    }

    #[test]
    fn match_from_character_classes() -> Result<(), PatternError> {
        // Pattern "[abc]+" on input "abccba" → Some(6)
        let ptrn = &[Quantifier::OneOrMore(Atom::Chars(
            vec![Atom::Literal('a'), Atom::Literal('b'), Atom::Literal('c')],
            true,
        ))];
        assert_eq!(Pattern::match_from(ptrn, "abccba"), Some(6));
        assert_eq!(Pattern::match_from(ptrn, "abcxyz"), Some(3));

        Ok(())
    }
}
