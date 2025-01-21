use alloc::vec::Vec;

use super::Dynamic;

#[derive(Debug, Clone)]
pub struct DynamicList {
    inner: Vec<Dynamic>,
}

impl DynamicList {
    pub fn new() -> DynamicList {
        DynamicList { inner: Vec::new() }
    }

    pub fn with_capacity(size: usize) -> DynamicList {
        DynamicList {
            inner: Vec::with_capacity(size),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Dynamic> {
        self.inner.get(index)
    }

    pub fn push(&mut self, value: impl Into<Dynamic>) {
        self.inner.push(value.into());
    }

    pub fn insert(&mut self, index: usize, value: impl Into<Dynamic>) {
        self.inner.insert(index, value.into());
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: Into<Dynamic>> From<Vec<T>> for DynamicList {
    fn from(value: Vec<T>) -> Self {
        let mut out = DynamicList::with_capacity(value.len());
        for element in value {
            out.push(element.into());
        }
        out
    }
}

impl PartialEq for DynamicList {
    fn eq(&self, other: &Self) -> bool {
        if other.len() != self.len() {
            return false;
        }
        self.inner
            .iter()
            .zip(other.inner.iter())
            .map(|(lhs, rhs)| lhs == rhs)
            .all(|x| x)
    }
}
