use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use json::{JsonValue, number::Number, object::Object};

use crate::{
    result::{DataError, DataResult},
    serialization::{CodecOps, ListView, MapView},
};

use super::{ListViewMut, MapViewMut, OwnedMapView};

#[derive(Debug, Clone)]
pub struct JsonOps;

impl CodecOps<JsonValue> for JsonOps {
    fn create_double(&self, value: &f64) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn create_string(&self, value: &str) -> JsonValue {
        JsonValue::String(value.to_string())
    }

    fn create_boolean(&self, value: &bool) -> JsonValue {
        JsonValue::Boolean(*value)
    }

    fn create_list(&self, value: impl IntoIterator<Item = JsonValue>) -> JsonValue {
        let iter = value.into_iter();
        let mut vec = Vec::with_capacity(iter.size_hint().1.unwrap_or(0));
        for element in iter {
            vec.push(element.clone());
        }
        JsonValue::Array(vec)
    }

    fn create_map(
        &self,
        pairs: impl IntoIterator<Item = (alloc::string::String, JsonValue)>,
    ) -> JsonValue {
        let iter = pairs.into_iter();
        let mut obj = Object::with_capacity(iter.size_hint().1.unwrap_or(0));
        for (key, element) in iter {
            obj.insert(&key, element.clone());
        }
        JsonValue::Object(obj)
    }

    fn create_unit(&self) -> JsonValue {
        JsonValue::Object(Object::new())
    }

    fn get_double(&self, value: &JsonValue) -> crate::result::DataResult<f64> {
        match value {
            JsonValue::Number(number) => Ok((*number).into()),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn get_string(&self, value: &JsonValue) -> crate::result::DataResult<alloc::string::String> {
        match value {
            JsonValue::String(string) => Ok(string.clone()),
            _ => Err(DataError::unexpected_type("string")),
        }
    }

    fn get_boolean(&self, value: &JsonValue) -> crate::result::DataResult<bool> {
        match value {
            JsonValue::Boolean(boolean) => Ok(*boolean),
            _ => Err(DataError::unexpected_type("boolean")),
        }
    }

    fn get_list(
        &self,
        value: &JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::ListView<JsonValue>> {
        match value {
            JsonValue::Array(_) => Ok(JsonListView { inner: value }),
            _ => Err(DataError::unexpected_type("array")),
        }
    }

    fn get_list_mut(
        &self,
        value: &mut JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::ListViewMut<JsonValue>> {
        match value {
            JsonValue::Array(_) => Ok(JsonListViewMut { inner: value }),
            _ => Err(DataError::unexpected_type("array")),
        }
    }

    fn get_map(
        &self,
        value: &JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::MapView<JsonValue>> {
        match value {
            JsonValue::Object(_) => Ok(JsonObjectView { inner: value }),
            _ => Err(DataError::unexpected_type("object")),
        }
    }

    fn get_map_mut(
        &self,
        value: &mut JsonValue,
    ) -> crate::result::DataResult<impl crate::serialization::MapViewMut<JsonValue>> {
        match value {
            JsonValue::Object(_) => Ok(JsonObjectViewMut { inner: value }),
            _ => Err(DataError::unexpected_type("object")),
        }
    }

    fn get_unit(&self, value: &JsonValue) -> crate::result::DataResult<()> {
        let JsonValue::Object(object) = value else {
            return Err(DataError::unexpected_type("object"));
        };
        if object.is_empty() {
            Ok(())
        } else {
            Err(DataError::new_custom("object must have 0 fields"))
        }
    }

    fn create_float(&self, value: &f32) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn create_byte(&self, value: &i8) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn create_short(&self, value: &i16) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn create_int(&self, value: &i32) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn create_long(&self, value: &i64) -> JsonValue {
        JsonValue::Number(Number::from(*value))
    }

    fn get_float(&self, value: &JsonValue) -> DataResult<f32> {
        match value {
            JsonValue::Number(number) => Ok((*number).into()),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn get_byte(&self, value: &JsonValue) -> DataResult<i8> {
        match value {
            JsonValue::Number(number) => Ok(Into::<f64>::into(*number) as i8),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn get_short(&self, value: &JsonValue) -> DataResult<i16> {
        match value {
            JsonValue::Number(number) => Ok(Into::<f64>::into(*number) as i16),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn get_int(&self, value: &JsonValue) -> DataResult<i32> {
        match value {
            JsonValue::Number(number) => Ok(Into::<f64>::into(*number) as i32),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn get_long(&self, value: &JsonValue) -> DataResult<i64> {
        match value {
            JsonValue::Number(number) => Ok(Into::<f64>::into(*number) as i64),
            _ => Err(DataError::unexpected_type("number")),
        }
    }

    fn take_map(&self, value: JsonValue) -> DataResult<impl OwnedMapView<JsonValue>> {
        match value {
            JsonValue::Object(_) => Ok(OwnedJsonObjectView { inner: value }),
            _ => Err(DataError::unexpected_type("object")),
        }
    }
}

struct OwnedJsonObjectView {
    inner: JsonValue,
}

impl OwnedMapView<JsonValue> for OwnedJsonObjectView {
    fn get(&self, name: &str) -> DataResult<&JsonValue> {
        let JsonValue::Object(object) = &self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        match object.get(name) {
            Some(v) => Ok(v),
            None => Err(DataError::key_not_found(name)),
        }
    }

    fn take(&mut self, name: &str) -> DataResult<JsonValue> {
        let JsonValue::Object(object) = &mut self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        match object.remove(name) {
            Some(v) => Ok(v),
            None => Err(DataError::key_not_found(name)),
        }
    }

    fn set(&mut self, name: &str, value: JsonValue) {
        let JsonValue::Object(object) = &mut self.inner else {
            return;
        };
        object.insert(name, value);
    }

    fn entries(self) -> impl IntoIterator<Item = (String, JsonValue)> {
        let JsonValue::Object(object) = self.inner else {
            return Vec::new();
        };
        let coll = object
            .iter()
            .map(|x| (x.0.to_string(), x.1.clone()))
            .collect::<Vec<_>>();
        coll
    }
}

struct JsonObjectView<'a> {
    inner: &'a JsonValue,
}

impl MapView<JsonValue> for JsonObjectView<'_> {
    fn get(&self, name: &str) -> DataResult<&JsonValue> {
        let JsonValue::Object(object) = self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        match object.get(name) {
            Some(v) => Ok(v),
            None => Err(DataError::key_not_found(name)),
        }
    }

    fn keys(&self) -> Vec<String> {
        if let JsonValue::Object(object) = &self.inner {
            return object.iter().map(|x| x.0.into()).collect();
        };
        Vec::new()
    }
}

struct JsonObjectViewMut<'a> {
    inner: &'a mut JsonValue,
}

impl MapView<JsonValue> for JsonObjectViewMut<'_> {
    fn get(&self, name: &str) -> DataResult<&JsonValue> {
        let JsonValue::Object(object) = &self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        match object.get(name) {
            Some(v) => Ok(v),
            None => Err(DataError::key_not_found(name)),
        }
    }

    fn keys(&self) -> Vec<String> {
        if let JsonValue::Object(object) = &self.inner {
            return object.iter().map(|x| x.0.into()).collect();
        };
        Vec::new()
    }
}

impl MapViewMut<JsonValue> for JsonObjectViewMut<'_> {
    fn get_mut(&mut self, name: &str) -> DataResult<&mut JsonValue> {
        let JsonValue::Object(object) = self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        match object.get_mut(name) {
            Some(v) => Ok(v),
            None => Err(DataError::key_not_found(name)),
        }
    }

    fn set(&mut self, name: &str, value: JsonValue) {
        let JsonValue::Object(object) = self.inner else {
            return;
        };
        object.insert(name, value);
    }

    fn remove(&mut self, key: &str) -> DataResult<JsonValue> {
        let JsonValue::Object(object) = self.inner else {
            return Err(DataError::unexpected_type("object"));
        };
        object.remove(key).ok_or(DataError::key_not_found(key))
    }
}

struct JsonListView<'a> {
    inner: &'a JsonValue,
}

struct JsonListViewMut<'a> {
    inner: &'a mut JsonValue,
}

impl ListView<JsonValue> for JsonListView<'_> {
    fn get(&self, index: usize) -> crate::result::DataResult<&JsonValue> {
        let JsonValue::Array(array) = self.inner else {
            return Err(DataError::unexpected_type("Array"));
        };
        let len = array.len();
        match array.get(index) {
            Some(v) => Ok(v),
            None => Err(DataError::list_index_out_of_bounds(index, len)),
        }
    }

    fn into_iter(self) -> impl Iterator<Item = JsonValue> {
        let JsonValue::Array(array) = self.inner else {
            panic!();
        };
        array.clone().into_iter()
    }
}

impl ListViewMut<JsonValue> for JsonListViewMut<'_> {
    fn append(&mut self, value: JsonValue) {
        let JsonValue::Array(array) = self.inner else {
            return;
        };
        array.push(value);
    }

    fn get_mut(&mut self, index: usize) -> DataResult<&mut JsonValue> {
        let JsonValue::Array(array) = self.inner else {
            return Err(DataError::unexpected_type("Array"));
        };
        let len = array.len();
        array
            .get_mut(index)
            .ok_or(DataError::list_index_out_of_bounds(index, len))
    }
}

#[cfg(test)]
mod tests {

    use crate::serialization::{Codec, DefaultCodec};

    use super::JsonOps;

    #[test]
    fn simple_encode_decode() {
        let encoded = f64::codec().encode(&JsonOps, &10.0).unwrap();
        let decoded = f64::codec().decode(&JsonOps, &encoded).unwrap();
        assert_eq!(decoded, 10.0);
    }
}
