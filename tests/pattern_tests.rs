use codecrafters_grep::errors::PatternError;
use codecrafters_grep::pattern::{Atom, Pattern, Quantifier};

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

#[test]
fn match_once_or_none_qntf() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"dogs?")?;
    assert!(ptrn.is_match("dog"));
    assert!(ptrn.is_match("dogs"));
    assert!(!ptrn.is_match("dos"));
    assert!(!ptrn.is_match("cat"));

    let ptrn = Pattern::new(r"colou?r")?;
    assert!(ptrn.is_match("color"));
    assert!(ptrn.is_match("colour"));
    assert!(!ptrn.is_match("colouur"));

    let ptrn = Pattern::new(r"\d?")?;
    assert!(ptrn.is_match("5"));
    assert!(ptrn.is_match(""));
    assert!(ptrn.is_match("foo"));

    let ptrn = Pattern::new(r"ca?t")?;
    assert!(!ptrn.is_match("cag"));

    Ok(())
}
