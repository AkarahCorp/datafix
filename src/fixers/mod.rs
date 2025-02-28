use core::marker::PhantomData;

mod types;
pub use types::*;
mod rules;
pub use rules::*;

use crate::serialization::CodecOps;

pub trait TypeRewriteRule<OT, O: CodecOps<OT>> {
    fn fix_data(&self, ops: O, value: OT) -> OT;
    fn fix_type(&self, ty: Type) -> Type;

    fn and_then(self, other: impl TypeRewriteRule<OT, O>) -> impl TypeRewriteRule<OT, O>
    where
        Self: Sized,
    {
        AndThenRule {
            left: self,
            right: other,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use json::JsonValue;

    use crate::{
        fixers::{Type, TypeRewriteRule},
        serialization::{CodecOps, json::JsonOps},
    };

    use super::Rules;

    #[test]
    pub fn default_field() {
        let mut object = JsonValue::new_object();
        let _ = object.insert("x", 10);

        let rule = Rules::new_field("y", |_ctx| JsonValue::Number(20.into()), |_ctx| Type::Int);

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, {
            let mut obj = JsonValue::new_object();
            let _ = obj.insert("x", 10);
            let _ = obj.insert("y", 20);
            obj
        })
    }

    #[test]
    pub fn copy_field_with_context() {
        let mut object = JsonValue::new_object();
        let _ = object.insert("x", 10);

        let rule = Rules::new_field(
            "y",
            |ctx| match ctx {
                JsonValue::Object(object) => object.get("x").unwrap().clone(),
                _ => JsonValue::Null,
            },
            |_ctx| Type::Int,
        );

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, {
            let mut obj = JsonValue::new_object();
            let _ = obj.insert("x", 10);
            let _ = obj.insert("y", 10);
            obj
        })
    }

    #[test]
    pub fn and_then_rule() {
        let mut object = JsonValue::new_object();
        let _ = object.insert("x", 10);

        let rule = Rules::new_field("y", |_| JsonValue::Number(20.into()), |_| Type::Int).and_then(
            Rules::new_field("z", |_ctx| JsonValue::Number(30.into()), |_ctx| Type::Long),
        );

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, {
            let mut obj = JsonValue::new_object();
            let _ = obj.insert("x", 10);
            let _ = obj.insert("y", 20);
            let _ = obj.insert("z", 30);
            obj
        })
    }

    #[test]
    pub fn field_removal_rule() {
        let mut object = JsonValue::new_object();
        let _ = object.insert("x", 10);

        let rule = Rules::remove_field("x");

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, JsonValue::new_object())
    }

    #[test]
    pub fn nested_field_application() {
        let mut nested = JsonValue::new_object();
        let _ = nested.insert("a", 10);

        let mut object = JsonValue::new_object();
        let _ = object.insert("i", nested);

        let rule = Rules::apply_to_field(
            "i",
            Rules::new_field("b", |_ctx| JsonValue::Number(20.into()), |_ctx| Type::Int),
        );

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, {
            let mut nested = JsonValue::new_object();
            let _ = nested.insert("a", 10);
            let _ = nested.insert("b", 20);

            let mut object = JsonValue::new_object();
            let _ = object.insert("i", nested);

            object
        })
    }
}
