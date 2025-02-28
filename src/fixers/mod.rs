mod types;
use core::marker::PhantomData;

pub use types::*;
mod tv;
pub use tv::*;
mod rules;
pub use rules::*;

use crate::serialization::CodecOps;

pub trait TypeRewriteRule<OT, O: CodecOps<OT>> {
    fn fix_data(&self, ops: O, value: OT) -> OT;
    fn fix_type(&self, ty: Type) -> Type;

    fn fix_tyval(&self, ty_val: TyVal<OT, O>) -> TyVal<OT, O> {
        match ty_val {
            TyVal::Value(value, ops) => TyVal::Value(self.fix_data(ops.clone(), value), ops),
            TyVal::Type(ty) => TyVal::Type(self.fix_type(ty)),
        }
    }

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

        let rule = Rules::new_field("y", || JsonValue::Number(20.into()), || Type::Int);

        let fixed = JsonOps.repair(object, rule);
        assert_eq!(fixed, {
            let mut obj = JsonValue::new_object();
            let _ = obj.insert("x", 10);
            let _ = obj.insert("y", 20);
            obj
        })
    }

    #[test]
    pub fn and_then_rule() {
        let mut object = JsonValue::new_object();
        let _ = object.insert("x", 10);

        let rule = Rules::new_field("y", || JsonValue::Number(20.into()), || Type::Int).and_then(
            Rules::new_field("z", || JsonValue::Number(30.into()), || Type::Long),
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
            Rules::new_field("b", || JsonValue::Number(20.into()), || Type::Int),
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
