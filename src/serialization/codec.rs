use crate::dynamic::Dynamic;

pub trait Codec {
    fn to_dyn(&self) -> Dynamic;
    fn from_dyn(&self, value: Dynamic) -> Self;
}
