use crate::pattern::{Atom, Quantifier};
use std::{iter::Peekable, str::Chars};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("No class found after: `\\`")]
    NoClassFound,

    #[error("Haven't found closing `]`")]
    InvalidCharClass,

    #[error("Haven't found closing `)`")]
    InvalidGroup,
}

pub fn parse(input: &str) -> Result<Vec<Quantifier>, ParserError> {
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
                            let prim = parse_primitives(&mut input_chars, cc)?;
                            group.push(prim);
                        }
                    }
                }

                if !found_closing {
                    return Err(ParserError::InvalidGroup);
                }

                body.push(quantify(&mut input_chars, Atom::AltGroup(group)))
            }
            // primitives
            cc => {
                let mut prim = parse_primitives(&mut input_chars, cc)?;
                body.append(&mut prim);
            }
        }
    }

    Ok(body)
}

fn parse_primitives(
    input_chars: &mut Peekable<Chars<'_>>,
    cc: char,
) -> Result<Vec<Quantifier>, ParserError> {
    let mut body = vec![];
    let mut maybe_cc = Some(cc);

    while let Some(curr_char) = maybe_cc {
        match curr_char {
            '.' => {
                let atom = Atom::Any;
                body.push(quantify(input_chars, atom))
            }
            // class
            '\\' => {
                if let Some(next_char) = input_chars.next() {
                    let atom = parse_atom(&next_char);
                    body.push(quantify(input_chars, atom))
                } else {
                    return Err(ParserError::NoClassFound);
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
                                class = parse_atom(&next_c);
                            } else {
                                class = parse_atom(&c);
                            }
                            char_class.push(class);
                        }
                        _ => char_class.push(Atom::Literal(c)),
                    }
                }

                if !found_closing {
                    return Err(ParserError::InvalidCharClass);
                }
                let atom = Atom::Seq(char_class, is_positive);
                body.push(quantify(input_chars, atom))
            }
            '^' => body.push(Quantifier::Exact(Atom::FromStart)),
            '$' => body.push(Quantifier::Exact(Atom::ToEnd)),
            // literal
            c => {
                let atom = Atom::Literal(c);
                body.push(quantify(input_chars, atom))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_negative_char_atom() -> Result<(), ParserError> {
        let ptrn = parse("[^abcd]")?;
        let char_atom = Atom::Seq(
            vec![
                Atom::Literal('a'),
                Atom::Literal('b'),
                Atom::Literal('c'),
                Atom::Literal('d'),
            ],
            false,
        );
        assert_eq!(ptrn.len(), 1);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(char_atom));

        Ok(())
    }

    #[test]
    fn parse_basic_char_atom() -> Result<(), ParserError> {
        let ptrn = parse(r"[abcde\d\w]")?;
        let char_atom = Atom::Seq(
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
        assert_eq!(ptrn.len(), 1);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(char_atom));

        Ok(())
    }

    #[test]
    fn parse_adv_char_atom() -> Result<(), ParserError> {
        let ptrn = parse(r"[abcde\d\w]\d\w322")?;
        let char_atom = Atom::Seq(
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
        assert_eq!(ptrn.len(), 6);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(char_atom));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.get(3).unwrap(), &Quantifier::Exact(Atom::Literal('3')));
        assert_eq!(ptrn.get(4).unwrap(), &Quantifier::Exact(Atom::Literal('2')));
        assert_eq!(ptrn.get(5).unwrap(), &Quantifier::Exact(Atom::Literal('2')));

        Ok(())
    }

    #[test]
    fn parse_word_char() -> Result<(), ParserError> {
        let ptrn = parse(r"\d\w\w\d")?;
        assert_eq!(ptrn.len(), 4);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(Atom::Digit));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::W));
        assert_eq!(ptrn.get(3).unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn parse_digit() -> Result<(), ParserError> {
        let input = r"\d";
        let ptrn = parse(input)?;
        assert_eq!(ptrn.len(), 1);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn parses_anychar() -> Result<(), ParserError> {
        let input = r"d.g";
        let ptrn = parse(input)?;
        assert_eq!(ptrn.len(), 3);
        assert_eq!(
            ptrn.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('g')));

        let input = r"...";
        let ptrn = parse(input)?;
        assert_eq!(ptrn.len(), 3);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Any));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Any));

        let input = r"d.?.+";
        let ptrn = parse(input)?;
        assert_eq!(ptrn.len(), 3);
        assert_eq!(
            ptrn.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::ZeroOrOne(Atom::Any));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::OneOrMore(Atom::Any));
        Ok(())
    }

    #[test]
    fn parse_iteral() -> Result<(), ParserError> {
        let input = r"abc123\d";
        let ptrn = parse(input)?;
        assert_eq!(ptrn.len(), 7);
        assert_eq!(
            ptrn.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('a'))
        );
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Literal('b')));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('c')));
        assert_eq!(ptrn.get(3).unwrap(), &Quantifier::Exact(Atom::Literal('1')));
        assert_eq!(ptrn.get(4).unwrap(), &Quantifier::Exact(Atom::Literal('2')));
        assert_eq!(ptrn.get(5).unwrap(), &Quantifier::Exact(Atom::Literal('3')));
        assert_eq!(ptrn.get(6).unwrap(), &Quantifier::Exact(Atom::Digit));

        Ok(())
    }

    #[test]
    fn parse_from_start_anchor() -> Result<(), ParserError> {
        let ptrn = parse(r"^test")?;
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(Atom::FromStart));

        Ok(())
    }

    #[test]
    fn parse_to_end_anchor() -> Result<(), ParserError> {
        let ptrn = parse(r"test$")?;
        assert_eq!(ptrn.last().unwrap(), &Quantifier::Exact(Atom::ToEnd));

        Ok(())
    }

    #[test]
    fn parse_from_start_to_end() -> Result<(), ParserError> {
        let ptrn = parse(r"^test$")?;
        assert_eq!(ptrn.first().unwrap(), &Quantifier::Exact(Atom::FromStart));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Literal('t')));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('e')));
        assert_eq!(ptrn.get(3).unwrap(), &Quantifier::Exact(Atom::Literal('s')));
        assert_eq!(ptrn.get(4).unwrap(), &Quantifier::Exact(Atom::Literal('t')));
        assert_eq!(ptrn.last().unwrap(), &Quantifier::Exact(Atom::ToEnd));

        Ok(())
    }

    #[test]
    fn parse_one_or_more_qntf() -> Result<(), ParserError> {
        let ptrn = parse(r"\d+")?;
        assert_eq!(ptrn.len(), 1);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::OneOrMore(Atom::Digit));

        let ptrn = parse(r"[abc1\d\w]+de")?;
        let char_atom = Atom::Seq(
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
        assert_eq!(ptrn.len(), 3);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::OneOrMore(char_atom));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Literal('d')));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('e')));

        let ptrn = parse(r"ca+ts")?;
        assert_eq!(ptrn.len(), 4);
        assert_eq!(
            ptrn.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('c'))
        );
        assert_eq!(
            ptrn.get(1).unwrap(),
            &Quantifier::OneOrMore(Atom::Literal('a'))
        );
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('t')));

        assert_eq!(ptrn.get(3).unwrap(), &Quantifier::Exact(Atom::Literal('s')));

        Ok(())
    }

    #[test]
    fn parse_once_or_not() -> Result<(), ParserError> {
        let ptrn = parse(r"dogs?")?;
        assert_eq!(ptrn.len(), 4);
        assert_eq!(
            ptrn.first().unwrap(),
            &Quantifier::Exact(Atom::Literal('d'))
        );

        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::Exact(Atom::Literal('o')));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::Exact(Atom::Literal('g')));
        assert_eq!(
            ptrn.get(3).unwrap(),
            &Quantifier::ZeroOrOne(Atom::Literal('s'))
        );

        let ptrn = parse(r"[abc]?\d?\w?")?;
        let char_atom = Atom::Seq(
            vec![Atom::Literal('a'), Atom::Literal('b'), Atom::Literal('c')],
            true,
        );
        assert_eq!(ptrn.len(), 3);
        assert_eq!(ptrn.first().unwrap(), &Quantifier::ZeroOrOne(char_atom));
        assert_eq!(ptrn.get(1).unwrap(), &Quantifier::ZeroOrOne(Atom::Digit));
        assert_eq!(ptrn.get(2).unwrap(), &Quantifier::ZeroOrOne(Atom::W));

        Ok(())
    }

    #[test]
    fn parse_alt_group() -> Result<(), ParserError> {
        let ptrn = parse(r"(c+at|dog?)([\dog]?|[\wod]+)?")?;
        assert_eq!(ptrn.len(), 2);
        assert_eq!(
            ptrn.first().unwrap(),
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
            ptrn.get(1).unwrap(),
            &Quantifier::ZeroOrOne(Atom::AltGroup(vec![
                vec![Quantifier::ZeroOrOne(Atom::Seq(
                    vec![Atom::Digit, Atom::Literal('o'), Atom::Literal('g')],
                    true
                ))],
                vec![Quantifier::OneOrMore(Atom::Seq(
                    vec![Atom::W, Atom::Literal('o'), Atom::Literal('d')],
                    true
                ))],
            ]))
        );
        let ptrn = parse(r"(cat|dog|\d\w)")?;
        assert_eq!(ptrn.len(), 1);
        assert_eq!(
            ptrn.first().unwrap(),
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
}
