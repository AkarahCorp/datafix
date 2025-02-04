use alloc::{string::String, vec::Vec};

use crate::result::DataResult;

/// A [`CodecOps`] represents a way of converting Rust values into the target datatype and vice-versa.
/// [`CodecOps`] is the recommended way to do this when interacting with [`Codec`].
///
/// This trait is very low-level. This is intended as an interface for developers making their own datatypes that
/// can interact with [`Codec`]s. For a developer simply wishing to be able to serialize & deserialize data,
/// the [`Codec`] trait is recommended instead.
///
/// Since fixing data is a big part of the [`Codec`] API, [`Codec::decode`] accepts a mutable reference. This is because when trying to update the value,
/// it will try to optimize the updating and apply it to the top-level instead of creating new copies everywhere.
///
/// [`Codec`]: [`super::Codec`]
pub trait CodecOps<T>: Clone {
    fn create_number(&self, value: &f64) -> T;
    fn create_string(&self, value: &str) -> T;
    fn create_boolean(&self, value: &bool) -> T;
    fn create_list(&self, value: impl IntoIterator<Item = T>) -> T;
    fn create_object(&self, pairs: impl IntoIterator<Item = (String, T)>) -> T;
    fn create_unit(&self) -> T;

    fn get_number(&self, value: &T) -> DataResult<f64>;
    fn get_string(&self, value: &T) -> DataResult<String>;
    fn get_boolean(&self, value: &T) -> DataResult<bool>;
    fn get_list(&self, value: &mut T) -> DataResult<impl ListView<T>>;
    fn get_object(&self, value: &mut T) -> DataResult<impl ObjectView<T>>;
    fn get_unit(&self, value: &T) -> DataResult<()>;

    // This purely exists for Optional Fields. The `Option` represents if a field is present,
    // the `DataResult` represents the actual field data.
    // TODO: convert to a no-copy implementation
    fn create_object_special(
        &self,
        pairs: impl IntoIterator<Item = Option<DataResult<(String, T)>>>,
    ) -> DataResult<T> {
        let iter1 = pairs.into_iter().filter_map(|x| x).filter_map(|x| x.ok());

        Ok(self.create_object(iter1))
    }
}

pub trait ObjectView<T> {
    fn get(&mut self, name: &str) -> DataResult<&mut T>;
    fn set(&mut self, name: &str, value: T);
    fn remove(&mut self, key: &str) -> DataResult<T>;
    fn keys(&self) -> Vec<String>;
    fn update<F: FnOnce(&mut T)>(&mut self, name: &str, f: F) {
        if let Ok(v) = self.get(name) {
            f(v)
        }
    }
}

pub trait ListView<T> {
    fn append(&mut self, value: T);
    fn get(&mut self, index: usize) -> DataResult<&mut T>;
    fn into_iter(self) -> impl Iterator<Item = T>;
}
