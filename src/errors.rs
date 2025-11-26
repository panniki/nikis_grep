use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatternError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("No class found after: `\\`")]
    NoClassFound,

    #[error("Haven't found closing `]`")]
    InvalidCharClass,

    #[error("Haven't found closing `)`")]
    InvalidGroup,
}
