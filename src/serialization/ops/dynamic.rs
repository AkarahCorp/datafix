use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    dynamic::{Dynamic, list::DynamicList, object::DynamicObject},
    result::{DataError, DataResult},
};

use super::CodecOps;

pub struct DynamicOps;

impl CodecOps<Dynamic> for DynamicOps {
    fn create_number(&self, value: &f64) -> Dynamic {
        Dynamic::Number(*value)
    }

    fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_string())
    }

    fn create_boolean(&self, value: &bool) -> Dynamic {
        Dynamic::Boolean(*value)
    }

    fn create_list<U: super::ConvertWithCodecOps<Dynamic>>(&self, value: &[U]) -> Dynamic {
        let mut list = DynamicList::new();
        for item in value {
            list.push(item.into_type(self));
        }
        Dynamic::List(list)
    }

    fn create_object<U: super::ConvertWithCodecOps<Dynamic>>(
        &self,
        pairs: &[(&str, U)],
    ) -> Dynamic {
        let mut obj = DynamicObject::new();
        for pair in pairs {
            obj.insert(pair.0, pair.1.into_type(self));
        }
        Dynamic::Object(obj)
    }

    fn get_number(&self, value: &Dynamic) -> DataResult<f64> {
        match value {
            Dynamic::Number(v) => Ok(*v),
            _ => Err(DataError::new("expected f64")),
        }
    }

    fn get_string(&self, value: &Dynamic) -> DataResult<String> {
        match value {
            Dynamic::String(v) => Ok(v.clone()),
            _ => Err(DataError::new("expected String")),
        }
    }

    fn get_boolean(&self, value: &Dynamic) -> DataResult<bool> {
        match value {
            Dynamic::Boolean(v) => Ok(v.clone()),
            _ => Err(DataError::new("expected bool")),
        }
    }

    fn get_list<U: super::ConvertWithCodecOps<Dynamic>>(
        &self,
        value: &Dynamic,
    ) -> DataResult<Vec<U>> {
        let Dynamic::List(value) = value else {
            return Err(DataError::new("expected List"));
        };
        let mut vec = Vec::new();
        for idx in 0..value.len() {
            let item = value.get(idx).unwrap();
            vec.push(U::from_type(self, item)?);
        }
        Ok(vec)
    }

    fn get_object<U: super::ConvertWithCodecOps<Dynamic>>(
        &self,
        value: &Dynamic,
    ) -> DataResult<BTreeMap<String, U>> {
        let Dynamic::Object(value) = value else {
            return Err(DataError::new("expected List"));
        };
        let mut map = BTreeMap::new();
        for key in value.keys() {
            let item = value.get(key).unwrap();
            map.insert(key.clone(), U::from_type(self, item)?);
        }
        Ok(map)
    }
}
