use std::marker::PhantomData;

use crate::{dynamic::Dynamic, result::DataResult};

use super::combinators::ListCodec;

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
}

pub trait DefaultCodec
where
    Self: Sized,
{
    fn codec() -> impl Codec<Self>;
}

#[cfg(test)]
mod tests {
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
}
