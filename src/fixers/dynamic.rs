use alloc::string::String;

use crate::{
    result::DataResult,
    serialization::{CodecOps, ListView, MapView},
};

/// An abstraction over a value and it's associated [`CodecOps`].
///
/// This holds a reference to an underlying value and owns an instance of [`CodecOps`], allowing for a more convenient transformation API
/// than using the [`CodecOps`] and underlying value directly.
pub struct Dynamic<'a, T, O: CodecOps<T>> {
    ops: O,
    value: &'a mut T,
}

impl<'a, T, O: CodecOps<T>> Dynamic<'a, T, O> {
    /// Creates a new instance of a [`Dynamic`], wrapping a value and a [`CodecOps`].
    pub fn new(ops: O, value: &mut T) -> Dynamic<T, O> {
        Dynamic { ops, value }
    }

    /// Returns a sharable reference to the underlying value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the mutable reference to the underlying value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consumes the [`Dynamic`], returning it's underlying values.
    pub fn into_inner(self) -> (O, &'a mut T) {
        (self.ops, self.value)
    }

    /// Returns a shared reference to the underlying [`CodecOps`].
    pub fn ops(&self) -> &O {
        &self.ops
    }

    /// Accepts a callback function that lets you modify the underlying value directly.
    pub fn mutate<F: FnOnce(&mut T)>(&mut self, f: F) {
        f(&mut self.value);
    }

    /// Attempts to convert the underlying value into an `f64`, returning an error variant if it's unable to do so.
    pub fn as_number(&self) -> DataResult<f64> {
        self.ops.get_number(&self.value)
    }

    /// Attempts to convert the underlying value into a `String`, returning an error variant if it's unable to do so.
    pub fn as_string(&self) -> DataResult<String> {
        self.ops.get_string(&self.value)
    }

    /// Attempts to convert the underlying value into a `bool`, returning an error variant if it's unable to do so.
    pub fn as_boolean(&self) -> DataResult<bool> {
        self.ops.get_boolean(&self.value)
    }

    /// Attempts to convert the underlying value into an empty tuple `()`, returning an error variant if it's unable to do so.
    pub fn as_unit(&self) -> DataResult<()> {
        self.ops.get_unit(&self.value)
    }

    /// Attempts to convert the underlying value into a reference to a map, returning an error variant if it's unable to do so.
    pub fn as_map(&mut self) -> DataResult<impl MapView<T>> {
        self.ops.get_map(self.value)
    }

    /// Attempts to convert the underlying value into a reference to a list, returning an error variant if it's unable to do so.
    pub fn as_list(&mut self) -> DataResult<impl ListView<T>> {
        self.ops.get_list(self.value)
    }
}
