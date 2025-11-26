use crate::errors::PatternError;

#[derive(Debug, PartialEq, Eq)]
enum Card {
    Digit,         // \d
    Literal(char), // abcdeAbcdzzz231237 etc
}

pub struct Pattern {
    expression: Vec<Card>,
}

impl Pattern {
    pub fn new(input: &str) -> Result<Self, PatternError> {
        let mut expression = vec![];
        let mut input_chars = input.chars().peekable();

        while let Some(curr_char) = input_chars.next() {
            match curr_char {
                // class
                '\\' => {
                    if let Some(next_char) = input_chars.next() {
                        let card = Self::parse_class(&next_char);
                        expression.push(card);
                    } else {
                        return Err(PatternError::NoClassFound);
                    }
                }
                '^' => unimplemented!("`^`"),
                '$' => unimplemented!("`$`"),
                '[' => unimplemented!("`[`"),
                // literal
                c => expression.push(Card::Literal(c)),
            }
        }

        Ok(Self { expression })
    }

    fn parse_class(c: &char) -> Card {
        match c {
            'd' => Card::Digit,
            '\\' => Card::Literal('\\'),
            another => panic!("not supported yet: {another}"),
        }
    }

    fn len(&self) -> usize {
        self.expression.len()
    }

    fn get(&self, index: usize) -> Option<&Card> {
        self.expression.get(index)
    }

    pub fn is_match(&self, input: &str) -> bool {
        if input.is_empty() || self.expression.is_empty() {
            return false;
        }

        for input_char in input.chars() {
            for card in self.expression.as_slice() {
                match card {
                    Card::Digit => {
                        if input_char.is_ascii_digit() {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    Card::Literal(literal) => {
                        if literal == &input_char {
                            return true;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn digit() -> Result<(), PatternError> {
        let input = r"\d";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.len(), 1);
        assert_eq!(ptrn.get(0).unwrap(), &Card::Digit);

        Ok(())
    }

    #[test]
    fn iteral() -> Result<(), PatternError> {
        let input = r"abc123\d";
        let ptrn = Pattern::new(input)?;
        assert_eq!(ptrn.len(), 7);
        assert_eq!(ptrn.get(0).unwrap(), &Card::Literal('a'));
        assert_eq!(ptrn.get(1).unwrap(), &Card::Literal('b'));
        assert_eq!(ptrn.get(2).unwrap(), &Card::Literal('c'));
        assert_eq!(ptrn.get(3).unwrap(), &Card::Literal('1'));
        assert_eq!(ptrn.get(4).unwrap(), &Card::Literal('2'));
        assert_eq!(ptrn.get(5).unwrap(), &Card::Literal('3'));
        assert_eq!(ptrn.get(6).unwrap(), &Card::Digit);

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
}
