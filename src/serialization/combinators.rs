use alloc::{string::ToString, vec::Vec};
use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};

use crate::{
    fixers::DataFixerRule,
    result::{DataError, DataResult},
};

use super::{Codec, ops::CodecOps};

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

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Vec<T>> {
        let list = ops.get_list(value)?;
        let mut vec = Vec::new();
        for item in list {
            vec.push(self.inner.decode(ops, &item)?);
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

    fn decode<OpsType, O: CodecOps<OpsType>>(&self, ops: &O, value: &OpsType) -> DataResult<U> {
        Ok((self.f)(&self.inner.decode(ops, value)?))
    }
}

pub(crate) struct DataFixCodec<T, C: Codec<T>, R: DataFixerRule> {
    pub(crate) inner: C,
    pub(crate) rule: R,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T, C: Codec<T>, R: DataFixerRule> Codec<T> for DataFixCodec<T, C, R> {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &T) -> DataResult<U> {
        let encoded = self.inner.encode(ops, value)?;
        let encoded = self.rule.fix(ops, &encoded);
        Ok(encoded)
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<T> {
        let value = self.rule.fix(ops, &value);
        self.inner.decode(ops, &value)
    }
}

pub(crate) struct PairCodec<L, R, Lc: Codec<L>, Rc: Codec<R>> {
    pub(crate) left: Lc,
    pub(crate) right: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (L, R)>,
}
impl<L, R, Lc: Codec<L>, Rc: Codec<R>> Codec<(L, R)> for PairCodec<L, R, Lc, Rc> {
    fn encode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &(L, R)) -> DataResult<U> {
        Ok(ops.create_object(
            [
                ("left".to_string(), self.left.encode(ops, &value.0)?),
                ("right".to_string(), self.right.encode(ops, &value.1)?),
            ]
            .into_iter(),
        ))
    }

    fn decode<U, O: super::ops::CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<(L, R)> {
        let obj = ops.get_object(value)?;
        let left = obj
            .get("left")
            .ok_or_else(|| DataError::new("Expected key \"left\" in pair"))?;
        let right = obj
            .get("right")
            .ok_or_else(|| DataError::new("Expected key \"right\" in pair"))?;
        Ok((self.left.decode(ops, left)?, self.right.decode(ops, right)?))
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
            Err(DataError::new(&alloc::format!(
                "Value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        } else {
            self.codec.encode(ops, value)
        }
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<T> {
        let decoded = self.codec.decode(ops, value)?;
        if self.range.contains(&decoded) {
            Ok(decoded)
        } else {
            Err(DataError::new(&alloc::format!(
                "Value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        }
    }
}
