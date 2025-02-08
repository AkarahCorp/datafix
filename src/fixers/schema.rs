use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use super::{Fixer, Type};

#[derive(Debug, Clone, PartialEq)]
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
    parent: Option<Box<Schema>>,
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
            parent: Some(Box::new(parent)),
        }
    }

    pub fn fixer<F: Fixer>(self, fixer: F) -> Schema {
        let mut schema = Schema {
            types: BTreeMap::new(),
            version: self.version.clone() + 1,
            parent: Some(Box::new(self)),
        };

        for tyr in schema.find_all_types() {
            let ty = schema.find_type(&tyr).unwrap();
            let mut diff_ty = ty.clone();
            fixer.fix_type(&tyr, &mut diff_ty);

            if ty != diff_ty {
                schema.insert_type(&tyr, diff_ty);
            }
        }

        schema
    }

    pub fn insert_type(&mut self, reference: &TypeReference, ty: Type) {
        self.types.insert(reference.name.clone(), ty);
    }

    pub fn insert_type_by_name(&mut self, name: &str, ty: Type) {
        self.types.insert(name.to_string(), ty);
    }

    pub fn find_type(&self, name: &TypeReference) -> Option<Type> {
        match self.types.get(&name.name) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.find_type_by_name(&name.name),
                None => None,
            },
        }
    }

    pub fn find_type_by_name(&self, name: &str) -> Option<Type> {
        match self.types.get(name) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.find_type_by_name(&name),
                None => None,
            },
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
        vec.dedup();
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
        schema.insert_type_by_name("TestType", Type::number());
        let mut schema = Schema::new(schema);
        schema.insert_type_by_name("TestType", Type::string());

        assert_eq!(
            schema.find_type_by_name("TestType").unwrap(),
            Type::string()
        );
    }
}
