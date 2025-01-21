pub mod combinators;
pub mod primitives;

use crate::{dynamic::Dynamic, result::DataResult};
use alloc::vec::Vec;
use combinators::{ListCodec, XMapCodec};
use core::marker::PhantomData;

pub trait Codec<T>
where
    Self: Sized,
{
    fn into_dyn(&self, value: T) -> DataResult<Dynamic>;
    fn from_dyn(&self, value: Dynamic) -> DataResult<T>;

    fn list_of(self) -> impl Codec<Vec<T>> {
        ListCodec {
            inner: self,
            _phantom: PhantomData,
        }
    }

    fn xmap<U, F, G>(self, to_new: F, from_new: G) -> impl Codec<U>
    where
        F: Fn(T) -> U,
        G: Fn(U) -> T,
    {
        XMapCodec {
            inner: self,
            f: to_new,
            g: from_new,
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

    use super::{Codec, DefaultCodec};

    #[test]
    fn f64_codec() {
        let value = 10.0;
        let encoded = f64::codec().into_dyn(value).unwrap();
        let decoded = f64::codec().from_dyn(encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn vec_codec() {
        let value = vec![10.0, 20.0, 30.0];

        let encoded = f64::codec().list_of().into_dyn(value.clone()).unwrap();
        let decoded = f64::codec().list_of().from_dyn(encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn xmap_codec() {
        let value: f32 = 15.0;

        let codec = f64::codec().xmap::<String, _, _>(
            |v| v.to_string(),
            |s| s.parse::<f64>().unwrap_or_else(|_| 0.0),
        );

        let encoded = codec.into_dyn(value.to_string()).unwrap();
        let decoded = codec.from_dyn(encoded).unwrap();

        assert_eq!(value.to_string(), decoded);
    }
}
