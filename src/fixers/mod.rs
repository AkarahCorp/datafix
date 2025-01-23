use crate::serialization::ops::CodecOps;

pub trait DataFixerRule {
    fn fix<T, O: CodecOps<T>>(&self, ops: &O, value: &T) -> T;
}
