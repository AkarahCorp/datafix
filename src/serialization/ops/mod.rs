use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};

use crate::result::DataResult;

pub mod dynamic;

pub trait CodecOps<T> {
    fn create_number(&self, value: &f64) -> T;
    fn create_string(&self, value: &str) -> T;
    fn create_boolean(&self, value: &bool) -> T;
    fn create_list<U: ConvertWithCodecOps<T>>(&self, value: &[U]) -> T;
    fn create_object<U: ConvertWithCodecOps<T>>(&self, pairs: &[(&str, U)]) -> T;

    fn get_number(&self, value: &T) -> DataResult<f64>;
    fn get_string(&self, value: &T) -> DataResult<String>;
    fn get_boolean(&self, value: &T) -> DataResult<bool>;
    fn get_list<U: ConvertWithCodecOps<T>>(&self, value: &T) -> DataResult<Vec<U>>;
    fn get_object<U: ConvertWithCodecOps<T>>(&self, value: &T) -> DataResult<BTreeMap<String, U>>;
}

pub trait ConvertWithCodecOps<T>
where
    Self: Sized,
{
    fn into_type(&self, ops: &impl CodecOps<T>) -> T;
    fn from_type(ops: &impl CodecOps<T>, value: &T) -> DataResult<Self>;
}

impl<T> ConvertWithCodecOps<T> for f64 {
    fn into_type(&self, ops: &impl CodecOps<T>) -> T {
        ops.create_number(self)
    }

    fn from_type(ops: &impl CodecOps<T>, value: &T) -> DataResult<Self> {
        ops.get_number(value)
    }
}

impl<T> ConvertWithCodecOps<T> for String {
    fn into_type(&self, ops: &impl CodecOps<T>) -> T {
        ops.create_string(self)
    }

    fn from_type(ops: &impl CodecOps<T>, value: &T) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl<T> ConvertWithCodecOps<T> for bool {
    fn into_type(&self, ops: &impl CodecOps<T>) -> T {
        ops.create_boolean(self)
    }

    fn from_type(ops: &impl CodecOps<T>, value: &T) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}
