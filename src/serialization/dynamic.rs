use super::CodecOps;

pub struct Dynamic<T, O: CodecOps<T>> {
    value: T,
    ops: O,
}

impl<T, O: CodecOps<T>> Dynamic<T, O> {
    pub fn new(value: T, ops: O) -> Dynamic<T, O> {
        Dynamic { value, ops }
    }

    pub fn ops(&self) -> O {
        self.ops.clone()
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}
