use crate::result::DataResult;

use super::{CodecOps, MapView, MapViewMut};

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

    pub fn get_byte(&self) -> DataResult<i8> {
        self.ops.get_byte(&self.value)
    }

    pub fn get_short(&self) -> DataResult<i16> {
        self.ops.get_short(&self.value)
    }

    pub fn as_int(&self) -> DataResult<i32> {
        self.ops.get_int(&self.value)
    }

    pub fn as_long(&self) -> DataResult<i64> {
        self.ops.get_long(&self.value)
    }

    pub fn as_float(&self) -> DataResult<f32> {
        self.ops.get_float(&self.value)
    }

    pub fn as_double(&self) -> DataResult<f64> {
        self.ops.get_double(&self.value)
    }

    pub fn create_byte(&self, value: i8) -> Self {
        Dynamic {
            value: self.ops.create_byte(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn create_short(&self, value: i16) -> Self {
        Dynamic {
            value: self.ops.create_short(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn create_int(&self, value: i32) -> Self {
        Dynamic {
            value: self.ops.create_int(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn create_long(&self, value: i64) -> Self {
        Dynamic {
            value: self.ops.create_long(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn create_float(&self, value: f32) -> Self {
        Dynamic {
            value: self.ops.create_float(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn create_double(&self, value: f64) -> Self {
        Dynamic {
            value: self.ops.create_double(&value),
            ops: self.ops.clone(),
        }
    }

    pub fn get_field(&self, key: &str) -> DataResult<Self> {
        Ok(Dynamic::new(
            self.ops.get_map(&self.value)?.get(key).cloned()?,
            self.ops(),
        ))
    }

    pub fn insert_field(&mut self, field: &str, value: T) -> DataResult<()> {
        self.ops.get_map_mut(&mut self.value)?.set(field, value);
        Ok(())
    }
}
