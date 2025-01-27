use crate::{dynamic::Dynamic, serialization::CodecOps};

pub trait Fixer {
    fn fix<T, O: CodecOps<T>>(&self, value: Dynamic<T, O>);
}
