use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};

use crate::result::DataResult;
mod dynamic;

/// A [`CodecOps`] represents a way of converting Rust values into the target datatype and vice-versa.
/// [`CodecOps`] is the recommended way to do this when interacting with [`Codec`].
///
/// This trait is very low-level. This is intended as an interface for developers making their own datatypes that
/// can interact with [`Codec`]s. For a developer simply wishing to be able to serialize & deserialize data,
/// the [`Codec`] trait is recommended instead.
///
/// [`Codec`]: [`super::Codec`]
pub trait CodecOps<T> {
    fn create_number(&self, value: &f64) -> T;
    fn create_string(&self, value: &str) -> T;
    fn create_boolean(&self, value: &bool) -> T;
    fn create_list(&self, value: &[T]) -> T;
    fn create_object(&self, pairs: &[(&str, T)]) -> T;
    fn create_unit(&self) -> T;

    fn get_number(&self, value: &T) -> DataResult<f64>;
    fn get_string(&self, value: &T) -> DataResult<String>;
    fn get_boolean(&self, value: &T) -> DataResult<bool>;
    fn get_list(&self, value: &T) -> DataResult<Vec<T>>;
    fn get_object(&self, value: &T) -> DataResult<BTreeMap<String, T>>;
    fn get_unit(&self, value: &T) -> DataResult<()>;
}
