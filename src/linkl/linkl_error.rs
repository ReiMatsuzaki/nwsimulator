use std::fmt;


#[derive(Debug)]
pub enum LinklError {
    NotEnoughBytes,
    InvalidBytes {msg: String},
}

impl fmt::Display for LinklError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinklError::NotEnoughBytes =>
                write!(f, "Not enough bytes"),
            LinklError::InvalidBytes {msg} =>
                write!(f, "Invalid Bytes:. {}", msg),
        }
    }
}

pub type Res<T> = Result<T, LinklError>;
