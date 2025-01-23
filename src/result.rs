use core::fmt::Debug;

use alloc::string::String;

pub struct DataError {
    message: String,
}

impl DataError {
    pub fn new(message: &str) -> DataError {
        DataError {
            message: message.into(),
        }
    }

    pub fn mark_ignorable(self) -> DataError {
        DataError {
            message: self.message,
        }
    }
}

impl Debug for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type DataResult<T> = Result<T, DataError>;
