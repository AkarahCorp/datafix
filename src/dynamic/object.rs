use super::Dynamic;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};

#[derive(Clone, Debug)]
pub struct DynamicObject {
    inner: BTreeMap<String, Dynamic>,
}

impl DynamicObject {
    pub fn new() -> DynamicObject {
        DynamicObject {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Dynamic> {
        self.inner.get_mut(key)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Dynamic>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn remove(&mut self, key: &str) -> Option<Dynamic> {
        self.inner.remove(key)
    }

    pub fn map<F: FnOnce(&Dynamic) -> Dynamic>(&mut self, key: &str, f: F) {
        if let Some(value) = self.get(key) {
            self.insert(key.to_string(), f(value));
        }
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }
}

impl PartialEq for DynamicObject {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.keys().ne(other.inner.keys()) {
            return false;
        }
        self.inner
            .keys()
            .map(|key| self.get(key) == other.get(key))
            .all(|x| x)
    }
}
