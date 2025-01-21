use crate::{
    dynamic::Dynamic,
    result::{DataError, DataResult},
};

use super::{Codec, DefaultCodec};

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

impl DefaultCodec for f64 {
    fn codec() -> impl Codec<Self> {
        F64Codec
    }
}
