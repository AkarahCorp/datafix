use alloc::{string::{String, ToString}, vec::Vec};
use json::{JsonValue, number::Number, object::Object};

use crate::{
    result::{DataError, DataResult},
    serialization::{CodecOps, ListView, ObjectView},
};

#[derive(Debug, Clone)]
pub struct JsonOps;

impl CodecOps<JsonValue> for JsonOps {
    fn create_number(&self, value: &f64) -> JsonValue {
        JsonValue::Number(Number::from(value.clone()))
    }

    fn create_string(&self, value: &str) -> JsonValue {
        JsonValue::String(value.to_string())
    }

    fn create_boolean(&self, value: &bool) -> JsonValue {
        JsonValue::Boolean(value.clone())
    }

    fn create_list(&self, value: impl IntoIterator<Item = JsonValue>) -> JsonValue {
        let iter = value.into_iter();
        let mut vec = Vec::with_capacity(iter.size_hint().1.unwrap_or_else(|| 0));
        for element in iter {
            vec.push(element.clone());
        }
        JsonValue::Array(vec)
    }

    fn create_object(
        &self,
        pairs: impl IntoIterator<Item = (alloc::string::String, JsonValue)>,
    ) -> JsonValue {
        let iter = pairs.into_iter();
        let mut obj = Object::with_capacity(iter.size_hint().1.unwrap_or_else(|| 0));
        for (key, element) in iter {
            obj.insert(&key, element.clone());
        }
        JsonValue::Object(obj)
    }

    fn create_unit(&self) -> JsonValue {
        JsonValue::Object(Object::new())
    }

    fn get_number(&self, value: &JsonValue) -> crate::result::DataResult<f64> {
        match value {
            JsonValue::Number(number) => Ok(number.clone().into()),
            _ => Err(DataError::new("")),
        }
    }

    fn get_string(&self, value: &JsonValue) -> crate::result::DataResult<alloc::string::String> {
        match value {
            JsonValue::String(string) => Ok(string.clone()),
            _ => Err(DataError::new("")),
        }
    }

    fn get_boolean(&self, value: &JsonValue) -> crate::result::DataResult<bool> {
        match value {
            JsonValue::Boolean(boolean) => Ok(boolean.clone()),
            _ => Err(DataError::new("")),
        }
    }

    fn get_list(
        &self,
        value: &mut JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::ListView<JsonValue>> {
        match value {
            JsonValue::Array(_) => Ok(JsonListView { inner: value }),
            _ => Err(DataError::new("")),
        }
    }

    fn get_object(
        &self,
        value: &mut JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::ObjectView<JsonValue>> {
        match value {
            JsonValue::Object(_) => Ok(JsonObjectView { inner: value }),
            _ => Err(DataError::new("")),
        }
    }

    fn get_unit(&self, value: &JsonValue) -> crate::result::DataResult<()> {
        let JsonValue::Object(object) = value else {
            return Err(DataError::new(""));
        };
        if object.len() == 0 {
            return Ok(());
        } else {
            return Err(DataError::new(""));
        }
    }
}

struct JsonObjectView<'a> {
    inner: &'a mut JsonValue,
}

impl<'a> ObjectView<JsonValue> for JsonObjectView<'a> {
    fn get(&mut self, name: &str) -> crate::result::DataResult<&mut JsonValue> {
        let JsonValue::Object(object) = self.inner else {
            return Err(DataError::new("message"));
        };
        match object.get_mut(name) {
            Some(v) => Ok(v),
            None => Err(DataError::new("err")),
        }
    }

    fn set(&mut self, name: &str, value: JsonValue) {
        if let JsonValue::Object(object) = self.inner {
            object.insert(name, value);
        }
    }

    fn keys(&self) -> Vec<String> {
        if let JsonValue::Object(object) = &self.inner {
            return object.iter().map(|x| x.0.into()).collect();
        };
        Vec::new()
    }
    
    fn remove(&mut self, key: &str) -> DataResult<JsonValue> {
        if let JsonValue::Object(object) = self.inner {
            return object.remove(key).ok_or_else(|| DataError::new("No key present"))
        }
        Err(DataError::new("Not an object"))
    }
}

struct JsonListView<'a> {
    inner: &'a mut JsonValue,
}

impl<'a> ListView<JsonValue> for JsonListView<'a> {
    fn append(&mut self, value: JsonValue) {
        if let JsonValue::Array(array) = self.inner {
            array.push(value);
        }
    }

    fn get(&mut self, index: usize) -> crate::result::DataResult<&mut JsonValue> {
        let JsonValue::Array(array) = self.inner else {
            return Err(DataError::new("?"));
        };
        match array.get_mut(index) {
            Some(v) => Ok(v),
            None => Err(DataError::new("?")),
        }
    }

    fn into_iter(self) -> impl Iterator<Item = JsonValue> {
        let JsonValue::Array(array) = self.inner else {
            panic!();
        };
        array.clone().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use json::{object::Object, JsonValue};

    use crate::{fixers::Fixers, serialization::{Codec, DefaultCodec, RecordCodecBuilder}};

    use super::JsonOps;

    #[test]
    fn simple_encode_decode() {
        let mut encoded = f64::codec().encode(&JsonOps, &10.0).unwrap();
        let decoded = f64::codec().decode(&JsonOps, &mut encoded).unwrap();
        assert_eq!(decoded, 10.0);
    }

    #[test]
    fn fixer_encode_decode() {
        #[derive(Debug, PartialEq)]
        struct SomeObject {
            id: i32,
            age: i32
        }

        impl SomeObject {
            pub fn id(&self) -> &i32 {
                &self.id
            }

            pub fn age(&self) -> &i32 {
                &self.age
            }

            pub fn new(id: i32, age: i32) -> SomeObject {
                SomeObject { id, age }
            }

            pub fn codec() -> impl Codec<SomeObject> {
                RecordCodecBuilder::new()
                    .field(i32::codec().field_of("id", SomeObject::id))
                    .field(i32::codec().field_of("age", SomeObject::age))
                    .build(SomeObject::new)
                    .fixer(Fixers::field_rename("old_id", "id"))
                    .fixer(Fixers::field_rename("old_age", "age"))
            }
        }

        let value = SomeObject::new(1, 20);
        let decoded = SomeObject::codec().decode(&JsonOps, &mut JsonValue::Object({
            let mut obj = Object::new();
            obj.insert("old_id", JsonValue::Number(1.into()));
            obj.insert("old_age", JsonValue::Number(20.into()));
            obj
        })).unwrap();
        assert_eq!(decoded, value);
    }
}
