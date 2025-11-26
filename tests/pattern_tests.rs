use codecrafters_grep::errors::PatternError;
use codecrafters_grep::pattern::Pattern;

#[test]
fn match_digit() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"\d")?;
    assert!(ptrn.is_match("3"));
    assert!(ptrn.is_match("12312412512"));
    assert!(!ptrn.is_match("nope"));

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
fn match_basic_ne_char_atom() -> Result<(), PatternError> {
    let ptrn = Pattern::new("[^abc]")?;
    assert!(ptrn.is_match("cat")); // cuz t not in the set.
    assert!(!ptrn.is_match("cab"));

    Ok(())
}

#[test]
fn match_sequence() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"\d apple")?;
    assert!(ptrn.is_match("1 apple"));
    assert!(!ptrn.is_match("1 orange"));

    Ok(())
}

#[test]
fn not_match_sequence_when_its_not_completed() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"\d \w\w\ws")?;
    assert!(!ptrn.is_match("sally has 1 dog"));
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
fn match_one_or_more_qntf() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"a+")?;
    assert!(!ptrn.is_match("dog"));
    assert!(ptrn.is_match("SaaS"));
    assert!(ptrn.is_match("SaS"));

    let ptrn = Pattern::new(r"ca+ts")?;
    assert!(ptrn.is_match("cats"));
    assert!(ptrn.is_match("caats"));
    assert!(!ptrn.is_match("cts"));

    let ptrn = Pattern::new(r"ca+ats")?;
    assert!(ptrn.is_match("caaats"));

    let ptrn = Pattern::new(r"^abc_\d+_xyz$")?;
    assert!(ptrn.is_match("abc_123_xyz"));

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

#[test]
fn match_any_char() -> Result<(), PatternError> {
    let ptrn = Pattern::new(r"d.g")?;
    assert!(ptrn.is_match("dog"));
    assert!(ptrn.is_match("dag"));
    assert!(ptrn.is_match("dig"));
    assert!(!ptrn.is_match("cog"));
    assert!(!ptrn.is_match("dg"));

    let ptrn = Pattern::new(r"...")?;
    assert!(ptrn.is_match("dog"));
    assert!(ptrn.is_match("cat"));
    assert!(!ptrn.is_match("\n"));

    let ptrn = Pattern::new(r".\d.")?;
    assert!(ptrn.is_match("a1b"));
    assert!(ptrn.is_match("113"));
    assert!(!ptrn.is_match("\n1b"));

    let ptrn = Pattern::new(r"g.+gol")?;
    assert!(ptrn.is_match("goøö0Ogol"));

    Ok(())
}

#[test]
fn match_alt_group() -> Result<(), PatternError> {
    let ptrn = Pattern::new("(cat|dog)")?;
    assert!(ptrn.is_match("dog"));
    assert!(ptrn.is_match("cat"));
    assert!(!ptrn.is_match("dag"));
    assert!(!ptrn.is_match("bag"));

    Ok(())
}
