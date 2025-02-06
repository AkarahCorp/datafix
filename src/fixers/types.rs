use core::fmt::Debug;

use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Unit,
    Map(TypeMap),
    Array(TypeArray),
}

impl Type {
    pub fn number() -> Type {
        Type::Number
    }

    pub fn string() -> Type {
        Type::String
    }

    pub fn boolean() -> Type {
        Type::Boolean
    }

    pub fn unit() -> Type {
        Type::Unit
    }

    pub fn map(map: TypeMap) -> Type {
        Type::Map(map)
    }

    pub fn array(array: Type) -> Type {
        Type::Array(TypeArray::new(array))
    }
}

#[derive(Clone, PartialEq)]
pub struct TypeMap {
    map: Option<BTreeMap<String, Type>>,
}

impl Debug for TypeMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.map {
            Some(map) => map.fmt(f)?,
            None => f.write_str("{?}")?,
        }
        Ok(())
    }
}
impl TypeMap {
    pub fn new() -> TypeMap {
        TypeMap { map: None }
    }

    pub fn insert_field(&mut self, key: &str, value: Type) {
        match &mut self.map {
            Some(map) => {
                map.insert(key.to_string(), value);
            }
            None => {
                let mut map = BTreeMap::new();
                map.insert(key.to_string(), value);
                self.map = Some(map)
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeArray {
    values: Box<Type>,
}

impl TypeArray {
    pub fn new(inner_type: Type) -> TypeArray {
        TypeArray {
            values: Box::new(inner_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::{
        fixers::{Type, TypeMap},
        serialization::{Codec, CodecAdapters, DefaultCodec, MapCodecBuilder},
    };

    struct User {
        name: String,
        age: i32,
    }

    impl User {
        pub fn name(&self) -> &String {
            &self.name
        }

        pub fn age(&self) -> &i32 {
            &self.age
        }

        pub fn new(name: String, age: i32) -> User {
            User { name, age }
        }

        pub fn codec() -> impl Codec<User> {
            MapCodecBuilder::new()
                .field(String::codec().field_of("name", User::name))
                .field(i32::codec().field_of("age", User::age))
                .build(User::new)
        }
    }

    #[test]
    pub fn simple_schema() {
        assert_eq!(
            Type::map({
                let mut map = TypeMap::new();
                map.insert_field("name", Type::string());
                map.insert_field("age", Type::number());
                map
            }),
            User::codec().get_type()
        )
    }
}
