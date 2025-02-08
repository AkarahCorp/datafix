use alloc::string::String;

use crate::serialization::MapView;

use super::{Fixer, Type, TypeReference};

pub struct FieldRenameFixer {
    old_name: String,
    new_name: String,
    targetting: TypeReference,
}

impl Fixer for FieldRenameFixer {
    fn fix_data<T, O: crate::serialization::CodecOps<T>>(
        &self,
        mut data: super::Dynamic<'_, T, O>,
        _ops: O,
    ) {
        if let Ok(mut map) = data.as_map() {
            let old_value = map.remove(&self.old_name).unwrap();
            map.set(&self.new_name, old_value);
        }
    }

    fn fix_type(&self, type_name: &TypeReference, input: &mut Type) {
        if *type_name == self.targetting {
            let Type::Map(map) = input else { return };
            let old_type = map.remove_field(&self.old_name).unwrap();
            map.insert_field(&self.new_name, old_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::fixers::{Schema, Type, TypeMap, TypeReference};

    use super::FieldRenameFixer;

    #[test]
    fn test_field_rename_type() {
        let mut schema = Schema::new_root();
        schema.insert_type_by_name(
            "SomeObject",
            Type::map({
                let mut map = TypeMap::new();
                map.insert_field("user_id", Type::number());
                map
            }),
        );
        let schema = schema.fixer(FieldRenameFixer {
            old_name: "user_id".to_string(),
            new_name: "user_number".to_string(),
            targetting: TypeReference {
                name: "SomeObject".to_string(),
            },
        });
        assert_eq!(
            schema.find_type_by_name("SomeObject"),
            Some(Type::map({
                let mut map = TypeMap::new();
                map.insert_field("user_number", Type::number());
                map
            }))
        );
    }
}
