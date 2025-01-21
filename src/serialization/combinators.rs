use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::{
    dynamic::{Dynamic, list::DynamicList},
    fixers::DataFixerRule,
    result::{DataError, DataResult},
};

use super::Codec;

pub struct ListCodec<T, C: Codec<T>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T, C: Codec<T>> Codec<Vec<T>> for ListCodec<T, C> {
    fn into_dyn(&self, value: Vec<T>) -> DataResult<Dynamic> {
        let mut list = DynamicList::new();
        for element in value {
            list.push(self.inner.into_dyn(element)?);
        }
        Ok(Dynamic::List(list))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<Vec<T>> {
        let Dynamic::List(list) = value else {
            return Err(DataError::new("expected a List"));
        };

        let mut vector = Vec::new();
        for idx in 0..list.len() {
            let item = list.get(idx).unwrap();
            vector.push(self.inner.from_dyn(item.clone())?);
        }
        Ok(vector)
    }
}

pub struct XMapCodec<T, U, C, F, G>
where
    C: Codec<T>,
    F: Fn(T) -> U,
    G: Fn(U) -> T,
{
    pub(crate) inner: C,
    pub(crate) f: F,
    pub(crate) g: G,
    pub(crate) _phantom: PhantomData<fn() -> (T, U)>,
}

impl<T, U, C, F, G> Codec<U> for XMapCodec<T, U, C, F, G>
where
    C: Codec<T>,
    F: Fn(T) -> U,
    G: Fn(U) -> T,
{
    fn into_dyn(&self, value: U) -> DataResult<Dynamic> {
        self.inner.into_dyn((self.g)(value))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<U> {
        Ok((self.f)(self.inner.from_dyn(value)?))
    }
}

pub struct DataFixCodec<T, C: Codec<T>, R: DataFixerRule> {
    pub(crate) inner: C,
    pub(crate) rule: R,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T, C: Codec<T>, R: DataFixerRule> Codec<T> for DataFixCodec<T, C, R> {
    fn into_dyn(&self, value: T) -> DataResult<Dynamic> {
        let mut dynamic = self.inner.into_dyn(value)?;
        self.rule.fix_dyn(&mut dynamic);
        Ok(dynamic)
    }

    fn from_dyn(&self, mut value: Dynamic) -> DataResult<T> {
        self.rule.fix_dyn(&mut value);
        self.inner.from_dyn(value)
    }
}
