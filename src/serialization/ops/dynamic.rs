use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    dynamic::{Dynamic, DynamicList, DynamicObject},
    result::{DataError, DataResult},
};

use super::CodecOps;

pub struct DynamicOps;

impl Dynamic {
    pub fn ops() -> impl CodecOps<Dynamic> {
        DynamicOps
    }
}

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

    fn create_list(&self, value: impl Iterator<Item = Dynamic>) -> Dynamic {
        let mut list = DynamicList::new();
        for element in value {
            list.push(element.clone());
        }
        Dynamic::List(list)
    }

    fn create_object(&self, pairs: impl Iterator<Item = (String, Dynamic)>) -> Dynamic {
        let mut obj = DynamicObject::new();
        for pair in pairs {
            obj.insert(pair.0, pair.1.clone());
        }
        Dynamic::Object(obj)
    }

    fn get_number(&self, value: &Dynamic) -> DataResult<f64> {
        match value {
            Dynamic::Number(v) => Ok(*v),
            _ => Err(DataError::new("Expected f64")),
        }
    }

    fn get_string(&self, value: &Dynamic) -> DataResult<String> {
        match value {
            Dynamic::String(v) => Ok(v.clone()),
            _ => Err(DataError::new("Expected String")),
        }
    }

    fn get_boolean(&self, value: &Dynamic) -> DataResult<bool> {
        match value {
            Dynamic::Boolean(v) => Ok(v.clone()),
            _ => Err(DataError::new("Expected bool")),
        }
    }

    fn get_list(&self, value: &Dynamic) -> DataResult<Vec<Dynamic>> {
        let Dynamic::List(value) = value else {
            return Err(DataError::new("Expected a valid list"));
        };
        let mut vec = Vec::new();
        for idx in 0..value.len() {
            vec.push(value.get(idx).unwrap().clone());
        }
        Ok(vec)
    }

    fn get_object(&self, value: &Dynamic) -> DataResult<BTreeMap<String, Dynamic>> {
        let Dynamic::Object(value) = value else {
            return Err(DataError::new("Expected a valid object"));
        };
        let mut map = BTreeMap::new();
        for key in value.keys() {
            let value = value.get(key);
            map.insert(key.clone(), value.unwrap().clone());
        }
        Ok(map)
    }

    fn create_unit(&self) -> Dynamic {
        Dynamic::Unit
    }

    fn get_unit(&self, value: &Dynamic) -> DataResult<()> {
        match value {
            Dynamic::Unit => Ok(()),
            _ => Err(DataError::new("Expected Unit")),
        }
    }
}
