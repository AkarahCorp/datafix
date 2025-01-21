use std::fmt::Debug;

pub struct DataError {
    message: String,
}

impl DataError {
    pub fn new(message: &str) -> DataError {
        DataError {
            message: message.into(),
        }
    }
}

impl Debug for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type DataResult<T> = Result<T, DataError>;
