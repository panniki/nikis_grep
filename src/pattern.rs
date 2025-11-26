use crate::errors::PatternError;
use std::{iter::Peekable, slice::Iter, str::Chars};

#[derive(Debug, PartialEq, Eq)]
pub enum Quantifier {
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
pub enum Atom {
    FromStart,              // ^
    ToEnd,                  // $
    Digit,                  // \d
    W,                      // \w
    Literal(char),          // abcdeAbcdzzz231237
    Chars(Vec<Atom>, bool), // [foo322]
    Any,                    // .
}

pub struct Pattern {
    pub body: Vec<Quantifier>,
}

impl Pattern {
    pub fn new(input: &str) -> Result<Self, PatternError> {
        let mut body = vec![];
        let mut input_chars = input.chars().peekable();

        while let Some(curr_char) = input_chars.next() {
            match curr_char {
                '.' => {
                    let atom = Atom::Any;
                    body.push(Self::quantify(&mut input_chars, atom))
                }
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
        if self.body.is_empty() {
            return false;
        }

        let mut found = false;
        let mut allow_unmatch = true;
        let mut i = 0;
        let mut chars_iter = input.chars().peekable();
        let mut expr_iter = self.body.iter().peekable();
        let mut maybe_matcher = expr_iter.next();
        let mut maybe_inp_char = chars_iter.next();

        // Handle empty input & ?
        if maybe_inp_char.is_none() {
            if let Some(Quantifier::ZeroOrOne(_)) = maybe_matcher {
                return expr_iter.len() == 0;
            } else {
                return false;
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
                        | Atom::Chars(_, _) => Self::match_atom(&inp_char, atom),
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
                        // if we match it here it means that input is longer and does not
                        // match the regex.
                        Atom::ToEnd => false,
                    },
                    Quantifier::OneOrMore(atom) => {
                        Self::count(&inp_char, atom, &mut chars_iter, &mut expr_iter) >= 1
                    }
                    Quantifier::ZeroOrOne(atom) => {
                        let mtch =
                            Self::count(&inp_char, atom, &mut chars_iter, &mut expr_iter) <= 1;

                        // handles case when regex longer than input
                        // ex: /colou?r/ -> "color"
                        if mtch && chars_iter.peek().is_none() {
                            maybe_matcher = expr_iter.next();
                        }

                        mtch
                    }
                };
                if found {
                    allow_unmatch = false;
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
        } else if let Some(Quantifier::ZeroOrOne(_)) = maybe_matcher {
            true
        } else {
            maybe_matcher.is_none()
        };

        found && is_final
    }

    pub fn count(
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
                _ => break,
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
            Atom::Any => in_char != &'\n',
            _ => unimplemented!(),
        }
    }
}
