use core::{
    error::Error,
    fmt::{Debug, Display},
};

use alloc::string::String;

use crate::serialization::Context;

pub enum DataError {
    UnexpectedType { expected: String },
    KeyNotFoundInMap { key: String },
    ListIndexOutOfBounds { list_length: usize, index: usize },
    Custom { message: String },
}

impl DataError {
    pub fn new_custom(message: &str) -> DataError {
        DataError::Custom {
            message: message.into(),
        }
    }

    pub fn unexpected_type(expected: &str) -> DataError {
        DataError::UnexpectedType {
            expected: expected.into(),
        }
    }

    pub fn key_not_found(key: &str) -> DataError {
        DataError::KeyNotFoundInMap { key: key.into() }
    }

    pub fn list_index_out_of_bounds(index: usize, list_length: usize) -> DataError {
        DataError::ListIndexOutOfBounds { list_length, index }
    }
}

impl Error for DataError {}

impl Display for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DataError::UnexpectedType { expected } => write!(f, "Expected type {}", expected),
            DataError::KeyNotFoundInMap { key } => write!(f, "Expected key {} in map", key),
            DataError::ListIndexOutOfBounds { list_length, index } => write!(
                f,
                "List index {} out of bounds for length {}",
                index, list_length
            ),
            DataError::Custom { message } => write!(f, "{}", message),
        }
    }
}

impl Debug for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Data Error: ")?;
        Display::fmt(self, f)
    }
}

pub type DataResult<T> = Result<T, DataError>;

pub struct CodecError {
    error: DataError,
    span: Context,
}

impl CodecError {
    pub fn new(error: DataError, span: Context) -> Self {
        CodecError { error, span }
    }

    pub fn error(&self) -> &DataError {
        &self.error
    }

    pub fn span(&self) -> &Context {
        &self.span
    }
}

impl Debug for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "An error has occured: \n{:?}\n{:?}",
            self.error, self.span
        )
    }
}

pub type CodecResult<T> = Result<T, CodecError>;
