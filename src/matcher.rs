use crate::pattern::{Atom, Quantifier};
use std::{iter::Peekable, slice::Iter, str::Chars};

pub fn match_from(pattern: &[Quantifier], input: &str) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    // dbg!(&pattern, &input, '-');

    let mut consumed = 0;
    let mut found = false;
    let mut allow_unmatch = true;
    let mut i = 0;
    // TODO: switch to &[char] instead of chars().peekable(), and iterate recursively
    let mut chars_iter = input.chars().peekable();
    let mut ptrn_iter = pattern.iter().peekable();
    let mut maybe_matcher = ptrn_iter.next();
    let mut maybe_inp_char = chars_iter.next();

    // Handles empty input & ?
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
                    Atom::Digit | Atom::W | Atom::Literal(_) | Atom::Any | Atom::Chars(_, _) => {
                        let mtch = match_atom(&inp_char, atom);
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
                    Atom::AltGroup(atom_list) => {
                        // TODO: fix the whole iteration algorithm.
                        let mut remaining: String = chars_iter.clone().collect();
                        if let Some(c) = maybe_inp_char {
                            remaining.insert(0, c);
                        }

                        for exp in atom_list {
                            if let Some(exp_consumed) = match_from(exp, &remaining) {
                                for _ in 0..exp_consumed {
                                    chars_iter.next();
                                }
                                consumed += exp_consumed;
                                found = true;
                                break;
                            } else {
                                found = false;
                            }
                        }

                        found
                    }
                },
                Quantifier::OneOrMore(atom) => {
                    let count = count(&inp_char, atom, &mut chars_iter, &mut ptrn_iter);
                    consumed += count;
                    count >= 1
                }
                Quantifier::ZeroOrOne(atom) => {
                    let count = count(&inp_char, atom, &mut chars_iter, &mut ptrn_iter);
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
            dbg!(&matcher, &inp_char, found, "->");
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

fn match_atom(in_char: &char, atom: &Atom) -> bool {
    match atom {
        Atom::Digit => in_char.is_ascii_digit(),
        Atom::Literal(literal) => literal == in_char,
        Atom::W => in_char.is_ascii_digit() || in_char.is_ascii_alphabetic() || in_char == &'_',
        Atom::Chars(cc, pos) => {
            let mtch = cc.iter().any(|c| match_atom(in_char, c));

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

fn count(
    curr_char: &char,
    curr_atom: &Atom,
    chars: &mut Peekable<Chars<'_>>,
    expr: &mut Peekable<Iter<Quantifier>>,
) -> usize {
    if !match_atom(curr_char, curr_atom) {
        return 0;
    }
    let mut counter = 1;

    while let Some(peek) = chars.peek() {
        let maybe_next_atom = expr.peek().map(|q| q.get_atom());
        match (match_atom(peek, curr_atom), maybe_next_atom) {
            (true, Some(next_atom)) => {
                if match_atom(peek, next_atom) && next_atom != curr_atom {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_basic_atom() {
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
        assert!(match_atom(&'g', &char_atom));
        assert!(!match_atom(&'z', &char_atom));
        assert!(match_atom(&'p', &char_atom));
    }

    #[test]
    fn match_from_basic_literals() {
        // Pattern "cat" on input "cat" → Some(3)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(match_from(ptrn, "cat"), Some(3));
        assert_eq!(match_from(ptrn, "dog"), None);

        // Pattern "do" on input "dog" → Some(2)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('d')),
            Quantifier::Exact(Atom::Literal('o')),
        ];
        assert_eq!(match_from(ptrn, "dog"), Some(2));
    }

    #[test]
    fn match_from_one_or_more_quantifier() {
        // Pattern "c+at" on input "ccat" → Some(4)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(match_from(ptrn, "ccat"), Some(4));
        assert_eq!(match_from(ptrn, "cccccat"), Some(7));

        // Pattern "\d+" on input "12345abc" → Some(5)
        let ptrn = &[Quantifier::OneOrMore(Atom::Digit)];
        assert_eq!(match_from(ptrn, "12345abc"), Some(5));
    }

    #[test]
    fn match_from_zero_or_one_quantifier() {
        // Pattern "colou?r" on input "color" → Some(5)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('o')),
            Quantifier::Exact(Atom::Literal('l')),
            Quantifier::Exact(Atom::Literal('o')),
            Quantifier::ZeroOrOne(Atom::Literal('u')),
            Quantifier::Exact(Atom::Literal('r')),
        ];
        assert_eq!(match_from(ptrn, "color"), Some(5));
        assert_eq!(match_from(ptrn, "colour"), Some(6));

        // Pattern "\d?" on input "foo" → Some(0)
        let ptrn = &[Quantifier::ZeroOrOne(Atom::Digit)];
        assert_eq!(match_from(ptrn, "foo"), Some(0));
    }

    #[test]
    fn match_from_multiple_quantifiers() {
        // Pattern "a+b+c" on input "aaabbbccc" → Some(9)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::OneOrMore(Atom::Literal('b')),
            Quantifier::OneOrMore(Atom::Literal('c')),
        ];
        assert_eq!(match_from(ptrn, "aaabbbccc"), Some(9));

        // Pattern "\d+\w+" on input "123abc" → Some(6)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Digit),
            Quantifier::OneOrMore(Atom::W),
        ];
        assert_eq!(match_from(ptrn, "123abc"), Some(6));
    }

    #[test]
    fn match_from_greedy_quantifiers() {
        // Pattern "a+a" on input "aaa" → Some(3)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('a')),
        ];
        assert_eq!(match_from(ptrn, "aaa"), Some(3));
        assert_eq!(match_from(ptrn, "aa"), Some(2));
        assert_eq!(match_from(ptrn, "a"), None);
    }

    #[test]
    fn match_from_anchors() {
        // Pattern "^cat" tests
        let ptrn = &[
            Quantifier::Exact(Atom::FromStart),
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        assert_eq!(match_from(ptrn, "cat"), Some(3));
        assert_eq!(match_from(ptrn, "dog cat"), None);

        // Pattern "cat$" tests
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
            Quantifier::Exact(Atom::ToEnd),
        ];
        assert_eq!(match_from(ptrn, "cat"), Some(3));
        assert_eq!(match_from(ptrn, "dog cat"), Some(3));
    }

    #[test]
    fn match_from_character_classes() {
        // Pattern "[abc]+" on input "abccba" → Some(6)
        let ptrn = &[Quantifier::OneOrMore(Atom::Chars(
            vec![Atom::Literal('a'), Atom::Literal('b'), Atom::Literal('c')],
            true,
        ))];
        assert_eq!(match_from(ptrn, "abccba"), Some(6));
        assert_eq!(match_from(ptrn, "abcxyz"), Some(3));
    }

    #[test]
    fn match_from_only_alt_group() {
        let ptrn = &[Quantifier::Exact(Atom::AltGroup(vec![
            vec![
                Quantifier::Exact(Atom::Literal('c')),
                Quantifier::Exact(Atom::Literal('a')),
                Quantifier::Exact(Atom::Literal('t')),
            ],
            vec![
                Quantifier::Exact(Atom::Literal('d')),
                Quantifier::Exact(Atom::Literal('o')),
                Quantifier::Exact(Atom::Literal('g')),
            ],
        ]))];

        assert_eq!(match_from(ptrn, "dog"), Some(3));
        assert_eq!(match_from(ptrn, "cat"), Some(3));
        assert_eq!(match_from(ptrn, "dat"), None);
        assert_eq!(match_from(ptrn, "a cog"), None);
    }

    #[test]
    fn match_from_include_alt_group() {
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal(' ')),
            Quantifier::Exact(Atom::AltGroup(vec![
                vec![
                    Quantifier::Exact(Atom::Literal('c')),
                    Quantifier::Exact(Atom::Literal('a')),
                    Quantifier::Exact(Atom::Literal('t')),
                ],
                vec![
                    Quantifier::Exact(Atom::Literal('d')),
                    Quantifier::Exact(Atom::Literal('o')),
                    Quantifier::Exact(Atom::Literal('g')),
                ],
            ])),
        ];
        assert_eq!(match_from(ptrn, "a cog"), None);
    }

    // #[test]
    // fn match_from_adv_alt_group_cases() -> Result<(), PatternError> {
    //     // Pattern: '^I see \d+ (cat|dog)s?$', match  on this "I see 2 dog3"
    //     let ptrn = &[
    //         Quantifier::Exact(Atom::FromStart),
    //         Quantifier::Exact(Atom::Literal('I')),
    //         Quantifier::Exact(Atom::Literal(' ')),
    //         Quantifier::Exact(Atom::Literal('s')),
    //         Quantifier::Exact(Atom::Literal('e')),
    //         Quantifier::Exact(Atom::Literal('e')),
    //         Quantifier::Exact(Atom::Literal(' ')),
    //         Quantifier::OneOrMore(Atom::Digit),
    //         Quantifier::Exact(Atom::Literal(' ')),
    //         Quantifier::Exact(Atom::AltGroup(vec![
    //             vec![
    //                 Quantifier::Exact(Atom::Literal('c')),
    //                 Quantifier::Exact(Atom::Literal('a')),
    //                 Quantifier::Exact(Atom::Literal('t')),
    //             ],
    //             vec![
    //                 Quantifier::Exact(Atom::Literal('d')),
    //                 Quantifier::Exact(Atom::Literal('o')),
    //                 Quantifier::Exact(Atom::Literal('g')),
    //             ],
    //         ])),
    //         Quantifier::ZeroOrOne(Atom::Literal('s')),
    //         Quantifier::Exact(Atom::Literal('?')),
    //         Quantifier::Exact(Atom::ToEnd),
    //     ];
    //     assert_eq!(match_from(ptrn, "I see 2 dog3"), None);
    //     Ok(())
    // }
}
