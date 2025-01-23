use core::marker::PhantomData;

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

pub trait F64Convertable
where
    Self: Sized + Copy,
{
    fn into_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! impl_f64_convertable {
    ($($t:ty),*) => {
        $(
            impl F64Convertable for $t {
                fn into_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(value: f64) -> Self {
                    value as $t
                }
            }

            impl DefaultCodec for $t {
                fn codec() -> impl Codec<Self> {
                    NumberCodec {
                        _phantom: PhantomData,
                    }
                }
            }
        )*
    };
}

impl_f64_convertable! { i8, i16, i32, i64, u8, u16, u32, u64, f32, usize, isize }

pub struct NumberCodec<N: F64Convertable> {
    _phantom: PhantomData<fn() -> N>,
}

impl<N: F64Convertable> Codec<N> for NumberCodec<N> {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &N) -> DataResult<U> {
        Ok(ops.create_number(&value.into_f64()))
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<N> {
        Ok(N::from_f64(ops.get_number(value)?))
    }
}
