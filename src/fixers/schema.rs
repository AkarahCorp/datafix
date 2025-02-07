use alloc::{
    collections::btree_map::BTreeMap,
    rc::Rc,
    string::{String, ToString},
};

use super::Type;

pub struct Schema {
    types: BTreeMap<String, Type>,
    version: u32,
    parent: Option<Rc<Schema>>,
}

impl Schema {
    pub fn new_root() -> Schema {
        Schema {
            types: BTreeMap::new(),
            version: 1,
            parent: None,
        }
    }

    pub fn new(parent: Schema) -> Schema {
        Schema {
            types: BTreeMap::new(),
            version: parent.version.clone() + 1,
            parent: Some(Rc::new(parent)),
        }
    }

    pub fn insert_type(&mut self, name: &str, ty: Type) {
        self.types.insert(name.to_string(), ty);
    }

    pub fn find_latest_type(&self, name: &str) -> Option<Type> {
        match self.types.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.clone()?.find_latest_type(name),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::fixers::Type;

    use super::Schema;

    #[test]
    pub fn type_finds() {
        let mut schema = Schema::new_root();
        schema.insert_type("TestType", Type::number());
        let mut schema = Schema::new(schema);
        schema.insert_type("TestType", Type::string());

        assert_eq!(schema.find_latest_type("TestType").unwrap(), Type::string());
    }
}
