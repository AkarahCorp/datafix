use alloc::string::String;

use crate::serialization::ObjectView;

use super::Fixer;

pub struct FieldRename {
    old_name: String,
    new_name: String
}

impl Fixer for FieldRename {
    fn fix<T, O: crate::serialization::CodecOps<T>>(&self, mut value: crate::dynamic::Dynamic<T, O>) {
        let Ok(mut object) = value.as_object() else { return };
        if let Ok(old_field_value) = object.remove(&self.old_name) {
            object.set(&self.new_name, old_field_value);
        }
    }
}

pub struct Fixers;

impl Fixers {
    pub fn field_rename(from: &str, to: &str) -> impl Fixer {
        FieldRename {
            old_name: from.into(),
            new_name: to.into()
        }
    }
}

