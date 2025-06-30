use std::{fmt, io};

#[derive(Debug)]
pub enum WavError {
    Io(io::Error),
    InvalidFormat(&'static str),
    UnsupportedFormat(&'static str),
    Corrupted(&'static str),
}

// This allows ? on I/O functions to work
impl From<io::Error> for WavError {
    fn from(err: io::Error) -> Self {
        WavError::Io(err)
    }
}

// For displaying the error when printing
impl std::fmt::Display for WavError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WavError::Io(e) => write!(f, "I/O error: {}", e),
            WavError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            WavError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            WavError::Corrupted(msg) => write!(f, "Corrupted data: {}", msg),
        }
    }
}

// For sourcing the original IO error from WavError
impl std::error::Error for WavError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let WavError::Io(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

pub type Result<T> = std::result::Result<T, WavError>;
