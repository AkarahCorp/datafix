mod builtins;
mod ops;

use alloc::{string::String, vec::Vec};
use builtins::{
    codecs::{BoundedCodec, ListCodec, PairCodec, XMapCodec},
    records::{OptionalField, RecordField},
};
use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};

pub use ops::*;

use crate::result::DataResult;
pub use builtins::record_builder::MapCodecBuilder;

/// A [`Codec<T>`] describes transformations to and from [`Dynamic`] for a type `T`.
/// [`Codec`]s are lazy, they don't do anything by themselves.
/// You need to call [`Codec::encode`], [`Codec::decode`] to change between `T` and [`Dynamic`].
/// For more complex use cases, you can call helper methods on [`CodecAdapter`] such as [`CodecAdapter::list_of`] and [`CodecAdapter::xmap`].
///
/// For implementors, try to keep implementations of this trait pure, immutable, and deterministic.
///
/// [`Dynamic`]: [`dynamic::Dynamic`]
/// [`CodecAdapter`]: [`serialization::CodecAdapter`]
/// [`CodecAdapter::xmap`]: [`serialization::CodecAdapter::xmap`]
/// [`CodecAdapter::list_of`]: [`serialization::CodecAdapter::list_of`]
pub trait Codec<T>
where
    Self: Sized,
{
    /// Transform a value of type `T` into a `U` using the provided [`CodecOps`], optionally returning an error .
    /// For implementors, this function should be pure and have no side effects.
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &T) -> DataResult<U>;
    /// Transforms a `U` value into a type `T` using the provided [`CodecOps`], optionally returning an error.
    /// For implementors, this function should be pure and have no side effects.
    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<T>;
}

/// Holds the adapter functions for [`Codec`] to allow codecs to do things such as:
/// - Turn into record fields
/// - Convert between types
/// - Have a list codec that contains the type of the provided codec
pub trait CodecAdapters<T>
where
    Self: Sized + Codec<T>,
{
    /// Returns a codec of this type that is intended for a field of a record.
    fn field_of<Struct>(
        self,
        name: impl Into<String>,
        getter: fn(&Struct) -> &T,
    ) -> RecordField<T, Self, Struct> {
        RecordField {
            field_name: name.into(),
            getter,
            codec: self,
            _phantom: PhantomData,
        }
    }

    /// Returns a codec of an [`Option`] wrapping this type, that is intended for an optional field of a record.
    fn optional_field_of<Struct>(
        self,
        name: impl Into<String>,
        getter: fn(&Struct) -> &Option<T>,
    ) -> OptionalField<T, Self, Struct> {
        OptionalField {
            field_name: name.into(),
            getter,
            codec: self,
            _phantom: PhantomData,
        }
    }

    /// Returns a codec that is a list of this codec.
    fn list_of(self) -> impl Codec<Vec<T>> {
        ListCodec {
            inner: self,
            _phantom: PhantomData,
        }
    }

    /// Maps the output of this codec between 2 transformation functions.
    /// Implementors should hold the invariant of `F(G(x)) = x` such that the functions can be used to freely convert between the two types.
    fn xmap<U, F, G>(self, to_new: F, from_new: G) -> impl Codec<U>
    where
        F: Fn(&T) -> U,
        G: Fn(&U) -> T,
    {
        XMapCodec {
            inner: self,
            f: to_new,
            g: from_new,
            _phantom: PhantomData,
        }
    }

    /// This returns a Codec that represents a pair of two codecs.
    fn pair<R>(self, right: impl Codec<R>) -> impl Codec<(T, R)> {
        PairCodec {
            left: self,
            right,
            _phantom: PhantomData,
        }
    }

    /// This bounds the result of this codec in the range, returning an error if the value is not within the range.
    fn bounded(self, range: impl RangeBounds<T>) -> impl Codec<T>
    where
        T: PartialOrd + Debug,
    {
        BoundedCodec {
            codec: self,
            range,
            _phantom: PhantomData,
        }
    }
}

impl<T, C: Codec<T>> CodecAdapters<T> for C {}

/// This trait is the go-to trait for when you want to provide a [`Codec`] for a type. These should be used whenever possible.
/// Please keep try to keep your implementations const-safe as this function in a future version of Rust may be upgraded to a `const fn`.
pub trait DefaultCodec
where
    Self: Sized,
{
    /// Returns the default codec for a type.
    fn codec() -> impl Codec<Self>;
}
