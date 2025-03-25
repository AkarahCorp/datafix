use core::marker::PhantomData;

mod types;
pub use types::*;
mod rules;
pub use rules::*;

use crate::serialization::CodecOps;

pub trait TypeRewriteRule<O: CodecOps> {
    fn fix_data(&self, ops: O, value: O::T) -> O::T;
    fn fix_type(&self, ty: Type) -> Type;

    fn and_then(self, other: impl TypeRewriteRule<O>) -> impl TypeRewriteRule<O>
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

        let rule = Rules::new_field("y", |ctx| ctx.create_int(20), |_ctx| Type::Int);

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
            |ctx| ctx.get_field("x").unwrap_or(ctx.create_int(0)),
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

        let rule = Rules::new_field("y", |ctx| ctx.create_int(20), |_| Type::Int).and_then(
            Rules::new_field("z", |ctx| ctx.create_int(30), |_ctx| Type::Int),
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
            Rules::new_field("b", |ctx| ctx.create_int(20), |_ctx| Type::Int),
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
