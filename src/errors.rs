use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum PatternError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("No class found after: `\\`")]
    NoClassFound,
}
