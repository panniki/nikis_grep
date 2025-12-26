use crate::pattern::{Atom, Quantifier};

pub fn match_from(
    chars: &[char],
    pattern: &[Quantifier],
    pos: usize,
    allow_unmatched: bool,
) -> Option<usize> {
    if pattern.is_empty() {
        return Some(0);
    }

    if chars.is_empty() {
        match &pattern[0] {
            Quantifier::Exact(Atom::ToEnd) => return Some(0),
            Quantifier::ZeroOrOne(_) => {
                // Allow ZeroOrOne to proceed and match 0 chars
            }
            _ => return None,
        }
    }

    let consumed = match &pattern[0] {
        Quantifier::Exact(atom) => match atom {
            Atom::Digit | Atom::W | Atom::Literal(_) | Atom::Any | Atom::Seq(_, _) => {
                if match_atom(&chars[0], atom).is_some() {
                    match_from(&chars[1..], &pattern[1..], pos + 1, false)
                        .map(|consumed| 1 + consumed)
                } else if !allow_unmatched {
                    None
                } else {
                    match_from(&chars[1..], pattern, pos + 1, true)
                }
            }
            Atom::FromStart => {
                if pos == 0 {
                    match_from(chars, &pattern[1..], pos + 1, false)
                } else {
                    None
                }
            }
            Atom::ToEnd => {
                if chars.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
            Atom::AltGroup(alternatives) => {
                if alternatives.is_empty() {
                    return None;
                }

                let rest = pattern[1..].to_vec();
                let first_alt = &alternatives[0];
                let mut combined = first_alt.clone();
                combined.extend(rest);

                if let Some(result) = match_from(chars, combined.as_slice(), pos, false) {
                    return Some(result);
                }

                if let Some(next_alt) = alternatives.get(1) {
                    let rest = pattern[1..].to_vec();
                    let mut combined = next_alt.clone();
                    combined.extend(rest);
                    match_from(chars, combined.as_slice(), pos, false)
                } else {
                    None
                }
            }
        },
        Quantifier::OneOrMore(atom) => {
            let maybe_next = pattern.get(1).map(|q| q.get_atom());
            let consumed = count(chars, atom, maybe_next)?;
            if consumed >= 1 {
                let next_pos = pos + consumed;
                if let Some(next) = maybe_next {
                    // "a+a" on input "aaa"
                    if next == atom && consumed >= 2 {
                        Some(consumed)
                    } else {
                        match_from(&chars[consumed..], &pattern[1..], next_pos, false)
                            .map(|c| c + consumed)
                    }
                } else {
                    match_from(&chars[consumed..], &pattern[1..], next_pos, false)
                        .map(|c| c + consumed)
                }
            } else if allow_unmatched {
                match_from(&chars[1..], pattern, pos + 1, true)
            } else {
                None
            }
        }
        Quantifier::ZeroOrOne(atom) => {
            let maybe_next = pattern.get(1).map(|q| q.get_atom());
            let consumed = count(chars, atom, maybe_next)?;
            if consumed <= 1 {
                let next_pos = pos + consumed;
                match_from(&chars[consumed..], &pattern[1..], next_pos, false).map(|c| c + consumed)
            } else if allow_unmatched {
                match_from(&chars[1..], pattern, pos + 1, true)
            } else {
                None
            }
        }
    };

    if !allow_unmatched && consumed.is_none() {
        None
    } else {
        consumed
    }
}

fn match_atom(in_char: &char, atom: &Atom) -> Option<usize> {
    let found = match atom {
        Atom::Digit => in_char.is_ascii_digit(),
        Atom::Literal(literal) => literal == in_char,
        Atom::W => in_char.is_ascii_digit() || in_char.is_ascii_alphabetic() || in_char == &'_',
        Atom::Seq(cc, pos) => {
            let mtch = cc.iter().any(|c| match_atom(in_char, c).is_some());

            if *pos {
                mtch
            } else {
                !mtch
            }
        }
        Atom::Any => in_char != &'\n',
        _ => false,
    };

    if found {
        Some(1)
    } else {
        None
    }
}

fn count(chars: &[char], current: &Atom, maybe_next: Option<&Atom>) -> Option<usize> {
    if chars.is_empty() || match_atom(&chars[0], current).is_none() {
        return Some(0);
    }

    // chars[0] matches current, we will consume it
    // Check if NEXT char (chars[1]) matches next pattern (lookahead)
    if chars.len() > 1 {
        if let Some(next) = maybe_next {
            if next != current && match_atom(&chars[1], next).is_some() {
                // Stop here, consume only chars[0], let next pattern have chars[1]
                return Some(1);
            }
        }
    }

    count(&chars[1..], current, maybe_next).map(|c| c + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_basic_atom() {
        assert_eq!(match_atom(&'4', &Atom::Digit), Some(1));
        assert_eq!(match_atom(&'f', &Atom::Digit), None);
        assert_eq!(match_atom(&'x', &Atom::Literal('x')), Some(1));
        assert_eq!(match_atom(&'y', &Atom::Literal('x')), None);
        assert_eq!(match_atom(&'w', &Atom::W), Some(1));
        assert_eq!(match_atom(&'1', &Atom::W), Some(1));
        assert_eq!(match_atom(&'!', &Atom::W), None);
        assert_eq!(match_atom(&'!', &Atom::Any), Some(1));
        assert_eq!(match_atom(&'3', &Atom::Any), Some(1));
        assert_eq!(match_atom(&'a', &Atom::Any), Some(1));
        assert_eq!(match_atom(&'\n', &Atom::Any), None);

        let seq = Atom::Seq(vec![Atom::Literal('g'), Atom::Digit, Atom::W], true);
        assert_eq!(match_atom(&'g', &seq), Some(1));
        assert_eq!(match_atom(&'z', &seq), Some(1));
        assert_eq!(match_atom(&'!', &seq), None);
        assert_eq!(match_atom(&'3', &seq), Some(1));
    }

    #[test]
    fn count_basic_atom() {
        assert_eq!(
            count(
                &['a', 'a', 'b'],
                &Atom::Literal('a'),
                Some(&Atom::Literal('b'))
            ),
            Some(2)
        );
        assert_eq!(
            count(
                &['a', 'a', 'a'],
                &Atom::Literal('a'),
                Some(&Atom::Literal('b'))
            ),
            Some(3)
        );
        assert_eq!(count(&['a', 'a', 'a'], &Atom::Literal('a'), None), Some(3));
        assert_eq!(
            count(
                &['a', 'a', 'a'],
                &Atom::Literal('a'),
                Some(&Atom::Literal('a'))
            ),
            Some(3)
        );
    }

    #[test]
    fn match_from_basic_literals() {
        // Pattern "cat" on input "cat" → Some(3)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        let chars = "cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "dog".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);

        // Pattern "do" on input "dog" → Some(2)
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('d')),
            Quantifier::Exact(Atom::Literal('o')),
        ];
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(2));
    }

    #[test]
    fn match_from_one_or_more_quantifier() {
        // Pattern "c+at" on input "ccat" → Some(4)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
        ];
        let chars = "ccat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(4));
        let chars = "cccccat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(7));

        // Pattern "\d+" on input "12345abc" → Some(5)
        let ptrn = &[Quantifier::OneOrMore(Atom::Digit)];
        let chars = "12345abc".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(5));
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
        let chars = "color".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(5));
        let chars = "colour".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(6));

        // Pattern "\d?" on input "foo" → Some(0)
        let ptrn = &[Quantifier::ZeroOrOne(Atom::Digit)];
        let chars = "foo".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(0));
    }

    #[test]
    fn match_from_multiple_quantifiers() {
        // Pattern "a+b+c" on input "aaabbbccc" → Some(9)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::OneOrMore(Atom::Literal('b')),
            Quantifier::OneOrMore(Atom::Literal('c')),
        ];
        let chars = "aaabbbccc".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(9));

        // Pattern "\d+\w+" on input "123abc" → Some(6)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Digit),
            Quantifier::OneOrMore(Atom::W),
        ];
        let chars = "123abc".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(6));
    }

    #[test]
    fn match_from_greedy_quantifiers() {
        // Pattern "a+a" on input "aaa" → Some(3)
        let ptrn = &[
            Quantifier::OneOrMore(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('a')),
        ];
        let chars = "aaa".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "aa".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(2));
        let chars = "a".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
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
        let chars = "cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "dog cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
        //
        // // Pattern "cat$" tests
        let ptrn = &[
            Quantifier::Exact(Atom::Literal('c')),
            Quantifier::Exact(Atom::Literal('a')),
            Quantifier::Exact(Atom::Literal('t')),
            Quantifier::Exact(Atom::ToEnd),
        ];
        let chars = "cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "dog cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
    }

    #[test]
    fn match_from_sequences() {
        // Pattern "[abc]+" on input "abccba" → Some(6)
        let ptrn = &[Quantifier::OneOrMore(Atom::Seq(
            vec![Atom::Literal('a'), Atom::Literal('b'), Atom::Literal('c')],
            true,
        ))];
        let chars = "abccba".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(6));
        let chars = "abcxyz".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
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

        let chars = "dog".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "cat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(3));
        let chars = "dat".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
        let chars = "a cog".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
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
        let chars = "a cog".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
    }

    #[test]
    fn match_from_adv_alt_group_cases() {
        // Pattern: '^I see \d+ (cat|dog)s?$', match  on this "I see 2 dog3"
        let ptrn = &[
            Quantifier::Exact(Atom::FromStart),
            Quantifier::Exact(Atom::Literal('I')),
            Quantifier::Exact(Atom::Literal(' ')),
            Quantifier::Exact(Atom::Literal('s')),
            Quantifier::Exact(Atom::Literal('e')),
            Quantifier::Exact(Atom::Literal('e')),
            Quantifier::Exact(Atom::Literal(' ')),
            Quantifier::OneOrMore(Atom::Digit),
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
            Quantifier::ZeroOrOne(Atom::Literal('s')),
            Quantifier::Exact(Atom::ToEnd),
        ];
        let chars = "I see 2 dog3".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), None);
        let chars = "I see 42 dogs".chars().collect::<Vec<_>>();
        assert_eq!(match_from(&chars, ptrn, 0, true), Some(13));
    }
}
