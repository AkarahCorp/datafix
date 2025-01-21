use std::marker::PhantomData;

use crate::{
    dynamic::{Dynamic, list::DynamicList},
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
