use crate::matcher;
use crate::parser;

// TODO: add ZeroOrMore *
#[derive(Debug, PartialEq, Eq)]
pub enum Quantifier {
    OneOrMore(Atom), // +
    ZeroOrOne(Atom), // ?
    Exact(Atom),
}

impl Quantifier {
    pub fn get_atom(&self) -> &Atom {
        match self {
            Self::Exact(atom) | Self::ZeroOrOne(atom) | Self::OneOrMore(atom) => atom,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Atom {
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

impl TryFrom<&str> for Pattern {
    type Error = parser::ParserError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(Pattern {
            body: parser::parse(input)?,
        })
    }
}

impl Pattern {
    pub fn is_match(&self, input: &str) -> bool {
        matcher::match_from(&self.body, input).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_digit() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"\d")?;
        assert!(ptrn.is_match("3"));
        assert!(ptrn.is_match("12312412512"));
        assert!(!ptrn.is_match("nope"));

        Ok(())
    }

    #[test]
    fn match_word_char() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"\w")?;
        assert!(ptrn.is_match(r"148"));
        assert!(ptrn.is_match(r"ORAnge"));
        assert!(ptrn.is_match(r"-+÷_-+="));

        Ok(())
    }

    #[test]
    fn invalid_char_atom() {
        assert!(Pattern::try_from("[abcd").is_err())
    }

    #[test]
    fn match_basic_char_atom() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from("[raspberry]")?;
        assert!(ptrn.is_match("p"));
        assert!(!ptrn.is_match(""));

        let ptrn = Pattern::try_from("[grape]")?;
        assert!(ptrn.is_match("gbc"));

        let ptrn = Pattern::try_from("[acdfghijk]")?;
        assert!(!ptrn.is_match("blueberry"));
        Ok(())
    }

    #[test]
    fn match_basic_ne_char_atom() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from("[^abc]")?;
        assert!(ptrn.is_match("cat")); // cuz t not in the set.
        assert!(!ptrn.is_match("cab"));

        Ok(())
    }

    #[test]
    fn match_sequence() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"\d apple")?;
        assert!(ptrn.is_match("1 apple"));
        assert!(!ptrn.is_match("1 orange"));

        Ok(())
    }

    #[test]
    fn not_match_sequence_when_its_not_completed() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"\d \w\w\ws")?;
        assert!(!ptrn.is_match("sally has 1 dog"));
        Ok(())
    }

    #[test]
    fn match_from_start() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"^log")?;
        assert!(ptrn.is_match("log"));

        let ptrn = Pattern::try_from(r"^log")?;
        assert!(ptrn.is_match("logs"));

        let ptrn = Pattern::try_from(r"^logs")?;
        assert!(!ptrn.is_match("slog"));

        let ptrn = Pattern::try_from(r"^\d\d\d")?;
        assert!(ptrn.is_match("123abc"));
        assert!(!ptrn.is_match("abc123"));

        Ok(())
    }

    #[test]
    fn match_to_end() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"dog$")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("hotdog"));
        assert!(!ptrn.is_match("dogs"));

        let ptrn = Pattern::try_from(r"\d\d\d$")?;
        assert!(ptrn.is_match("abc123"));

        let ptrn = Pattern::try_from(r"\w\w\w$")?;
        assert!(!ptrn.is_match("abc123@"));

        let ptrn = Pattern::try_from(r"\w\w\w$")?;
        assert!(!ptrn.is_match("abc123cde"));

        Ok(())
    }

    #[test]
    fn match_one_or_more_qntf() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"a+")?;
        assert!(!ptrn.is_match("dog"));
        assert!(ptrn.is_match("SaaS"));
        assert!(ptrn.is_match("SaS"));

        let ptrn = Pattern::try_from(r"ca+ts")?;
        assert!(ptrn.is_match("cats"));
        assert!(ptrn.is_match("caats"));
        assert!(!ptrn.is_match("cts"));

        let ptrn = Pattern::try_from(r"ca+ats")?;
        assert!(ptrn.is_match("caaats"));

        let ptrn = Pattern::try_from(r"^abc_\d+_xyz$")?;
        assert!(ptrn.is_match("abc_123_xyz"));

        Ok(())
    }

    #[test]
    fn match_once_or_none_qntf() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"dogs?")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("dogs"));
        assert!(!ptrn.is_match("dos"));
        assert!(!ptrn.is_match("cat"));

        let ptrn = Pattern::try_from(r"colou?r")?;
        assert!(ptrn.is_match("color"));
        assert!(ptrn.is_match("colour"));
        assert!(!ptrn.is_match("colouur"));

        let ptrn = Pattern::try_from(r"\d?")?;
        assert!(ptrn.is_match("5"));
        assert!(ptrn.is_match(""));
        assert!(ptrn.is_match("foo"));

        let ptrn = Pattern::try_from(r"ca?t")?;
        assert!(!ptrn.is_match("cag"));

        Ok(())
    }

    #[test]
    fn match_any_char() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from(r"d.g")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("dag"));
        assert!(ptrn.is_match("dig"));
        assert!(!ptrn.is_match("cog"));
        assert!(!ptrn.is_match("dg"));

        let ptrn = Pattern::try_from(r"...")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("cat"));
        assert!(!ptrn.is_match("\n"));

        let ptrn = Pattern::try_from(r".\d.")?;
        assert!(ptrn.is_match("a1b"));
        assert!(ptrn.is_match("113"));
        assert!(!ptrn.is_match("\n1b"));

        let ptrn = Pattern::try_from(r"g.+gol")?;
        assert!(ptrn.is_match("goøö0Ogol"));

        Ok(())
    }

    #[test]
    fn match_alt_group() -> Result<(), parser::ParserError> {
        let ptrn = Pattern::try_from("(cat|dog)")?;
        assert!(ptrn.is_match("dog"));
        assert!(ptrn.is_match("cat"));
        assert!(!ptrn.is_match("dag"));
        assert!(!ptrn.is_match("bag"));

        Ok(())
    }
}
