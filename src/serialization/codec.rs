use crate::dynamic::Dynamic;

pub trait Codec<T> {
    fn into_dyn(&self, value: T) -> Option<Dynamic>;
    fn from_dyn(&self, value: Dynamic) -> Option<T>;
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
}
