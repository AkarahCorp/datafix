use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    result::{DataError, DataResult},
    serialization::{Codec, CodecOps, DefaultCodec, ListView, MapView},
};

pub(crate) struct F64Codec;

impl Codec<f64> for F64Codec {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &f64) -> DataResult<U> {
        Ok(ops.create_number(value))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<f64> {
        ops.get_number(value)
    }
}

impl DefaultCodec for f64 {
    fn codec() -> impl Codec<Self> {
        F64Codec
    }
}

pub(crate) struct StringCodec;

impl Codec<String> for StringCodec {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &String) -> DataResult<U> {
        Ok(ops.create_string(value))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl DefaultCodec for String {
    fn codec() -> impl Codec<Self> {
        StringCodec
    }
}

pub(crate) struct BoolCodec;

impl Codec<bool> for BoolCodec {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &bool) -> DataResult<U> {
        Ok(ops.create_boolean(value))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}

impl DefaultCodec for bool {
    fn codec() -> impl Codec<Self> {
        BoolCodec
    }
}

pub(crate) trait F64Convertable
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

pub(crate) struct NumberCodec<N: F64Convertable> {
    _phantom: PhantomData<fn() -> N>,
}

impl<N: F64Convertable> Codec<N> for NumberCodec<N> {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &N) -> DataResult<U> {
        Ok(ops.create_number(&value.into_f64()))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<N> {
        Ok(N::from_f64(ops.get_number(value)?))
    }
}

pub(crate) struct ListCodec<T, C: Codec<T>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T, C: Codec<T>> Codec<Vec<T>> for ListCodec<T, C> {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Vec<T>) -> DataResult<U> {
        let mut list = Vec::new();
        for element in value {
            list.push(self.inner.encode(ops, element)?);
        }
        Ok(ops.create_list(list.into_iter()))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<Vec<T>> {
        let list = ops.get_list(value)?;
        let mut vec = Vec::new();
        for mut item in list.into_iter() {
            vec.push(self.inner.decode(ops, &mut item)?);
        }
        Ok(vec)
    }
}

pub(crate) struct XMapCodec<T, U, C, F, G>
where
    C: Codec<T>,
    F: Fn(&T) -> U,
    G: Fn(&U) -> T,
{
    pub(crate) inner: C,
    pub(crate) f: F,
    pub(crate) g: G,
    pub(crate) _phantom: PhantomData<fn() -> (T, U)>,
}

impl<T, U, C, F, G> Codec<U> for XMapCodec<T, U, C, F, G>
where
    C: Codec<T>,
    F: Fn(&T) -> U,
    G: Fn(&U) -> T,
{
    fn encode<U2, O: CodecOps<U2>>(&self, ops: &O, value: &U) -> DataResult<U2> {
        self.inner.encode(ops, &(self.g)(value))
    }

    fn decode<OpsType, O: CodecOps<OpsType>>(&self, ops: &O, value: &mut OpsType) -> DataResult<U> {
        Ok((self.f)(&self.inner.decode(ops, value)?))
    }
}

pub(crate) struct PairCodec<L, R, Lc: Codec<L>, Rc: Codec<R>> {
    pub(crate) left: Lc,
    pub(crate) right: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (L, R)>,
}
impl<L, R, Lc: Codec<L>, Rc: Codec<R>> Codec<(L, R)> for PairCodec<L, R, Lc, Rc> {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &(L, R)) -> DataResult<U> {
        Ok(ops.create_map(
            [
                ("left".to_string(), self.left.encode(ops, &value.0)?),
                ("right".to_string(), self.right.encode(ops, &value.1)?),
            ]
            .into_iter(),
        ))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<(L, R)> {
        let mut obj = ops.get_map(value)?;
        let mut left = obj.get("left")?;
        let p1 = self.left.decode(ops, &mut left)?;
        let mut right = obj.get("right")?;
        let p2 = self.right.decode(ops, &mut right)?;
        Ok((p1, p2))
    }
}

pub(crate) struct BoundedCodec<T: PartialOrd + Debug, C: Codec<T>, R: RangeBounds<T>> {
    pub(crate) codec: C,
    pub(crate) range: R,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T: PartialOrd + Debug, C: Codec<T>, R: RangeBounds<T>> Codec<T> for BoundedCodec<T, C, R> {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &T) -> DataResult<U> {
        if !self.range.contains(value) {
            Err(DataError::new_custom(&alloc::format!(
                "value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        } else {
            self.codec.encode(ops, value)
        }
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<T> {
        let decoded = self.codec.decode(ops, value)?;
        if self.range.contains(&decoded) {
            Ok(decoded)
        } else {
            Err(DataError::new_custom(&alloc::format!(
                "value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        }
    }
}
