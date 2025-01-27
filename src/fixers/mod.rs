use crate::serialization::CodecOps;

pub trait Fixer {
    fn fix<T, O: CodecOps<T>>(&self, ops: &O, value: &T) -> T;
}
