use crate::result::DataResult;

use super::{CodecOps, MapView};

#[derive(Debug, Clone)]
pub struct Dynamic<T: Clone, O: CodecOps<T>> {
    value: T,
    ops: O,
}

impl<T: Clone, O: CodecOps<T>> Dynamic<T, O> {
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

    pub fn create_int(&self, value: i32) -> Self {
        Dynamic {
            value: self.ops.create_int(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn get(&self, key: &str) -> DataResult<Self> {
        Ok(Dynamic::new(
            self.ops.get_map(&self.value)?.get(key).cloned()?,
            self.ops(),
        ))
    }
}
