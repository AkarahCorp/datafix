mod builder;
mod combinators;
mod ops;
mod primitives;
mod record;

use crate::{fixers::Fixer, result::DataResult};
use alloc::{string::String, vec::Vec};
use combinators::{BoundedCodec, DataFixCodec, ListCodec, PairCodec, XMapCodec};
use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};
use record::{OptionalField, RecordField};

pub use builder::RecordCodecBuilder;
pub use ops::CodecOps;

/// A [`Codec<T>`] describes transformations to and from [`Dynamic`] for a type `T`.
/// [`Codec`]s are lazy, they don't do anything by themselves.
/// You need to call [`Codec::encode`], [`Codec::decode`] to change between `T` and [`Dynamic`].
/// For more complex use cases, you can call helper methods such as [`Codec::list_of`] and [`Codec::xmap`].
///
/// For implementors, try to keep implementations of this trait pure, immutable, and deterministic.
///
/// [`Dynamic`]: [`dynamic::Dynamic`]
pub trait Codec<T>
where
    Self: Sized,
{
    /// Transform a value of type `T` into a `U` using the provided [`CodecOps`], optionally returning an error .
    /// For implementors, this function should be pure and have no side effects.
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &T) -> DataResult<U>;
    /// Transforms a `U` value into a type `T` using the provided [`CodecOps`], optionally returning an error.
    /// For implementors, this function should be pure and have no side effects.
    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<T>;

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

    fn list_of(self) -> impl Codec<Vec<T>> {
        ListCodec {
            inner: self,
            _phantom: PhantomData,
        }
    }

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

    fn pair<R>(self, right: impl Codec<R>) -> impl Codec<(T, R)> {
        PairCodec {
            left: self,
            right,
            _phantom: PhantomData,
        }
    }

    fn fixer<R: Fixer>(self, rule: R) -> impl Codec<T> {
        DataFixCodec {
            inner: self,
            rule,
            _phantom: PhantomData,
        }
    }

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

pub trait DefaultCodec
where
    Self: Sized,
{
    fn codec() -> impl Codec<Self>;
}

#[cfg(test)]
mod tests {
    use alloc::{
        string::{String, ToString},
        vec,
    };

    use crate::dynamic::Dynamic;

    use super::{Codec, DefaultCodec};

    #[test]
    fn f64_codec() {
        let value = 10.0;
        let encoded = f64::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = f64::codec().decode(&Dynamic::ops(), &encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn ints_codec() {
        let value = 10;
        let encoded = i32::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = i32::codec().decode(&Dynamic::ops(), &encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn bounded_codec() {
        let value = 10;
        let encoded = i32::codec()
            .bounded(-10..15)
            .encode(&Dynamic::ops(), &value);
        assert!(encoded.is_ok());

        let encoded = i32::codec()
            .bounded(15..300)
            .encode(&Dynamic::ops(), &value);
        assert!(encoded.is_err());
    }

    #[test]
    fn vec_codec() {
        let value = vec![10.0, 20.0, 30.0];

        let encoded = f64::codec()
            .list_of()
            .encode(&Dynamic::ops(), &value)
            .unwrap();
        let decoded = f64::codec()
            .list_of()
            .decode(&Dynamic::ops(), &encoded)
            .unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn xmap_codec() {
        let value: f32 = 15.0;

        let codec = f64::codec().xmap::<String, _, _>(
            |v| v.to_string(),
            |s| s.parse::<f64>().unwrap_or_else(|_| 0.0),
        );

        let encoded = codec.encode(&Dynamic::ops(), &value.to_string()).unwrap();
        assert_eq!(encoded, Dynamic::new(15.0));
        let decoded = codec.decode(&Dynamic::ops(), &encoded).unwrap();
        assert_eq!(value.to_string(), decoded);
    }

    #[test]
    fn pair_codec() {
        let codec = Codec::pair(f64::codec(), f64::codec());
        let encoded = codec.encode(&Dynamic::ops(), &(10.0, 20.0)).unwrap();
        let decoded = codec.decode(&Dynamic::ops(), &encoded).unwrap();
        assert_eq!((10.0, 20.0), decoded);
    }
}
