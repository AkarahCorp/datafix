use crate::dynamic::Dynamic;

use super::{Codec, DataError, DataResult, Primitives};

impl Primitives {
    pub const fn f64() -> impl Codec<f64> {
        F64Codec
    }
}

pub struct F64Codec;

impl Codec<f64> for F64Codec {
    fn into_dyn(&self, value: f64) -> DataResult<Dynamic> {
        Ok(Dynamic::Number(value))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<f64> {
        value
            .as_number()
            .copied()
            .ok_or_else(|| DataError::new("Expected f64"))
    }
}
