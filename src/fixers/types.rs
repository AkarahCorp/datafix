use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
};

use crate::result::{DataError, DataResult};

#[derive(Debug, Clone)]
pub enum Type {
    Byte,
    Short,
    Int,
    Long,

    Float,
    Double,

    String,

    Array(ArrayType),
    Object(ObjectType),
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    ty: Box<Type>,
}

impl ArrayType {
    pub fn new(ty: Type) -> Self {
        ArrayType { ty: Box::new(ty) }
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    fields: BTreeMap<String, Type>,
}

impl ObjectType {
    pub fn new() -> Self {
        ObjectType {
            fields: BTreeMap::new(),
        }
    }

    pub fn get(&self, field: &str) -> DataResult<Type> {
        self.fields
            .get(field)
            .ok_or(DataError::key_not_found(field))
            .cloned()
    }

    pub fn field(mut self, field: &str, ty: Type) -> Self {
        self.fields.insert(field.to_string(), ty);
        self
    }

    pub fn insert(&mut self, field: &str, ty: Type) {
        self.fields.insert(field.to_string(), ty);
    }
}

impl Default for ObjectType {
    fn default() -> Self {
        Self::new()
    }
}
