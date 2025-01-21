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

pub struct Primitives;

#[cfg(test)]
mod tests {
    use super::{Codec, Primitives};

    #[test]
    fn f64_codec() {
        let value = 10.0;
        let encoded = Primitives::f64().into_dyn(value).unwrap();
        let decoded = Primitives::f64().from_dyn(encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn vec_codec() {
        let value = vec![10.0, 20.0, 30.0];

        let encoded = Primitives::f64().list_of().into_dyn(value.clone()).unwrap();
        let decoded = Primitives::f64().list_of().from_dyn(encoded).unwrap();

        assert_eq!(value, decoded);
    }
}
