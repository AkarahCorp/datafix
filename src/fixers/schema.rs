use alloc::{
    collections::btree_map::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use super::{Fixer, Type};

pub struct TypeReference {
    pub(crate) name: String,
}

impl TypeReference {
    pub fn new(id: &str) -> TypeReference {
        TypeReference { name: id.into() }
    }
}

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

    pub fn fixer<F: Fixer>(self, fixer: F) -> Schema {
        let mut schema = Schema {
            types: BTreeMap::new(),
            version: self.version.clone() + 1,
            parent: Some(Rc::new(self)),
        };

        for tyr in schema.find_all_types() {
            let ty = schema.find_type(&tyr).unwrap();
            fixer.fix_type(&mut schema, tyr, ty);
        }

        schema
    }

    pub fn insert_type(&mut self, name: &str, ty: Type) {
        self.types.insert(name.to_string(), ty);
    }

    pub fn insert_type_ref(&mut self, reference: &TypeReference, ty: Type) {
        self.types.insert(reference.name.clone(), ty);
    }

    pub fn find_type(&self, name: &TypeReference) -> Option<Type> {
        match self.types.get(&name.name) {
            Some(value) => Some(value.clone()),
            None => self.parent.clone()?.find_latest_type(&name.name),
        }
    }

    pub fn find_latest_type(&self, name: &str) -> Option<Type> {
        match self.types.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.clone()?.find_latest_type(name),
        }
    }

    pub fn find_all_types(&self) -> Vec<TypeReference> {
        let mut vec = self
            .types
            .keys()
            .map(|x| TypeReference { name: x.clone() })
            .collect::<Vec<_>>();
        if let Some(parent) = &self.parent {
            vec.extend(parent.find_all_types());
        }
        vec
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
