use core::marker::PhantomData;

use alloc::string::{String, ToString};

use crate::serialization::{CodecOps, Dynamic, MapView, MapViewMut};

use super::{Type, TypeRewriteRule};

pub struct Rules;

impl Rules {
    pub fn new_field<O: CodecOps, F: Fn(&Dynamic<O>) -> Dynamic<O>, G: Fn(&Type) -> Type>(
        field_name: &str,
        value_function: F,
        type_function: G,
    ) -> impl TypeRewriteRule<O> {
        NewFieldRule {
            field_name: field_name.to_string(),
            value_function,
            type_function,
            _phantom: PhantomData,
        }
    }

    pub fn remove_field<O: CodecOps>(field_name: &str) -> impl TypeRewriteRule<O> {
        RemoveFieldRule {
            field_name: field_name.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn apply_to_field<O: CodecOps>(
        field_name: &str,
        rule: impl TypeRewriteRule<O>,
    ) -> impl TypeRewriteRule<O> {
        ApplyRuleToFieldRule {
            field_name: field_name.to_string(),
            rule,
            _phantom: PhantomData,
        }
    }
}

pub struct NewFieldRule<O: CodecOps, F: Fn(&Dynamic<O>) -> Dynamic<O>, G: Fn(&Type) -> Type> {
    field_name: String,
    value_function: F,
    type_function: G,
    _phantom: PhantomData<O>,
}

impl<O: CodecOps, F: Fn(&Dynamic<O>) -> Dynamic<O>, G: Fn(&Type) -> Type> TypeRewriteRule<O>
    for NewFieldRule<O, F, G>
{
    fn fix_data(&self, ops: O, value: O::T) -> O::T {
        let dynamic = Dynamic::new(value, ops.clone());
        let result = (self.value_function)(&dynamic);
        let mut value = dynamic.into_inner();
        if let Ok(mut obj) = ops.get_map_mut(&mut value) {
            obj.set(&self.field_name, result.into_inner());
        }
        value
    }

    fn fix_type(&self, mut ty: Type) -> Type {
        {
            let result = (self.type_function)(&ty);
            if let Type::Object(obj) = &mut ty {
                obj.insert(&self.field_name, result);
            }
        }
        ty
    }
}

pub struct AndThenRule<O: CodecOps, L: TypeRewriteRule<O>, R: TypeRewriteRule<O>> {
    pub(crate) left: L,
    pub(crate) right: R,
    pub(crate) _phantom: PhantomData<O>,
}

impl<O: CodecOps, L: TypeRewriteRule<O>, R: TypeRewriteRule<O>> TypeRewriteRule<O>
    for AndThenRule<O, L, R>
{
    fn fix_data(&self, ops: O, value: O::T) -> O::T {
        self.right
            .fix_data(ops.clone(), self.left.fix_data(ops, value))
    }

    fn fix_type(&self, ty: Type) -> Type {
        self.right.fix_type(self.left.fix_type(ty))
    }
}

pub struct ApplyRuleToFieldRule<O: CodecOps, R: TypeRewriteRule<O>> {
    field_name: String,
    rule: R,
    _phantom: PhantomData<O>,
}

impl<O: CodecOps, R: TypeRewriteRule<O>> TypeRewriteRule<O> for ApplyRuleToFieldRule<O, R> {
    fn fix_data(&self, ops: O, mut value: O::T) -> O::T {
        if ops.get_map(&value).is_ok() {
            let mut object = ops.get_map_mut(&mut value).unwrap();
            if let Ok(field_value) = object.get(&self.field_name) {
                object.set(
                    &self.field_name,
                    self.rule.fix_data(ops.clone(), field_value.clone()),
                );
            }
        }
        value
    }

    fn fix_type(&self, mut ty: Type) -> Type {
        if let Type::Object(object) = &mut ty {
            if let Ok(field) = object.remove(&self.field_name) {
                object.insert(&self.field_name, self.rule.fix_type(field));
            }
        }
        ty
    }
}

pub struct RemoveFieldRule<O: CodecOps> {
    field_name: String,
    _phantom: PhantomData<O>,
}

impl<O: CodecOps> TypeRewriteRule<O> for RemoveFieldRule<O> {
    fn fix_data(&self, ops: O, mut value: O::T) -> O::T {
        {
            if let Ok(mut obj) = ops.get_map_mut(&mut value) {
                let _ = obj.remove(&self.field_name);
            }
        }
        value
    }

    fn fix_type(&self, mut ty: Type) -> Type {
        {
            if let Type::Object(obj) = &mut ty {
                let _ = obj.remove(&self.field_name);
            }
        }
        ty
    }
}
