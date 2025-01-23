use alloc::string::String;

use crate::result::DataResult;

use super::{Codec, DefaultCodec};

pub struct F64Codec;

impl Codec<f64> for F64Codec {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &f64) -> DataResult<U> {
        Ok(ops.create_number(value))
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<f64> {
        ops.get_number(value)
    }
}

impl DefaultCodec for f64 {
    fn codec() -> impl Codec<Self> {
        F64Codec
    }
}

pub struct StringCodec;

impl Codec<String> for StringCodec {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &String) -> DataResult<U> {
        Ok(ops.create_string(value))
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl DefaultCodec for String {
    fn codec() -> impl Codec<Self> {
        StringCodec
    }
}

pub struct BoolCodec;

impl Codec<bool> for BoolCodec {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &bool) -> DataResult<U> {
        Ok(ops.create_boolean(value))
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}

impl DefaultCodec for bool {
    fn codec() -> impl Codec<Self> {
        BoolCodec
    }
}
