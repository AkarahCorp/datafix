use alloc::{format, string::String};

use crate::{
    dynamic::Dynamic,
    result::{DataError, DataResult},
};

use super::{Codec, DefaultCodec};

pub struct F64Codec;

impl Codec<f64> for F64Codec {
    fn into_dyn(&self, value: &f64) -> DataResult<Dynamic> {
        Ok(Dynamic::Number(*value))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<f64> {
        value
            .as_number()
            .copied()
            .ok_or_else(|| DataError::new(&format!("Expected f64, found {:?}", value)))
    }
}

impl DefaultCodec for f64 {
    fn codec() -> impl Codec<Self> {
        F64Codec
    }
}

pub struct StringCodec;

impl Codec<String> for StringCodec {
    fn into_dyn(&self, value: &String) -> DataResult<Dynamic> {
        Ok(Dynamic::String(value.clone()))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<String> {
        let Dynamic::String(str) = value else {
            return Err(DataError::new("expected String"));
        };
        Ok(str)
    }
}

impl DefaultCodec for String {
    fn codec() -> impl Codec<Self> {
        StringCodec
    }
}

pub struct BoolCodec;

impl Codec<bool> for BoolCodec {
    fn into_dyn(&self, value: &bool) -> DataResult<Dynamic> {
        Ok(Dynamic::Boolean(*value))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<bool> {
        let Dynamic::Boolean(bl) = value else {
            return Err(DataError::new("expected bool"));
        };
        Ok(bl)
    }
}

impl DefaultCodec for bool {
    fn codec() -> impl Codec<Self> {
        BoolCodec
    }
}
