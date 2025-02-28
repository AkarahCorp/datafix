use core::marker::PhantomData;

use alloc::string::{String, ToString};

use crate::serialization::{CodecOps, MapViewMut, OwnedMapView};

use super::{Type, TypeRewriteRule};

pub struct Rules;

impl Rules {
    pub fn new_field<OT: Clone, O: CodecOps<OT>, F: Fn(&OT) -> OT, G: Fn(&Type) -> Type>(
        field_name: &str,
        value_function: F,
        type_function: G,
    ) -> impl TypeRewriteRule<OT, O> {
        NewFieldRule {
            field_name: field_name.to_string(),
            value_function,
            type_function,
            _phantom: PhantomData,
        }
    }

    pub fn remove_field<OT: Clone, O: CodecOps<OT>>(
        field_name: &str,
    ) -> impl TypeRewriteRule<OT, O> {
        RemoveFieldRule {
            field_name: field_name.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn apply_to_field<OT: Clone, O: CodecOps<OT>>(
        field_name: &str,
        rule: impl TypeRewriteRule<OT, O>,
    ) -> impl TypeRewriteRule<OT, O> {
        ApplyRuleToFieldRule {
            field_name: field_name.to_string(),
            rule,
            _phantom: PhantomData,
        }
    }
}

pub struct NewFieldRule<OT: Clone, O: CodecOps<OT>, F: Fn(&OT) -> OT, G: Fn(&Type) -> Type> {
    field_name: String,
    value_function: F,
    type_function: G,
    _phantom: PhantomData<(OT, O)>,
}

impl<OT: Clone, O: CodecOps<OT>, F: Fn(&OT) -> OT, G: Fn(&Type) -> Type> TypeRewriteRule<OT, O>
    for NewFieldRule<OT, O, F, G>
{
    fn fix_data(&self, ops: O, mut value: OT) -> OT {
        {
            let result = (self.value_function)(&value);
            if let Ok(mut obj) = ops.get_map_mut(&mut value) {
                obj.set(&self.field_name, result);
            }
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

pub struct AndThenRule<
    OT: Clone,
    O: CodecOps<OT>,
    L: TypeRewriteRule<OT, O>,
    R: TypeRewriteRule<OT, O>,
> {
    pub(crate) left: L,
    pub(crate) right: R,
    pub(crate) _phantom: PhantomData<(OT, O)>,
}

impl<OT: Clone, O: CodecOps<OT>, L: TypeRewriteRule<OT, O>, R: TypeRewriteRule<OT, O>>
    TypeRewriteRule<OT, O> for AndThenRule<OT, O, L, R>
{
    fn fix_data(&self, ops: O, value: OT) -> OT {
        self.right
            .fix_data(ops.clone(), self.left.fix_data(ops, value))
    }

    fn fix_type(&self, ty: Type) -> Type {
        self.right.fix_type(self.left.fix_type(ty))
    }
}

pub struct ApplyRuleToFieldRule<OT: Clone, O: CodecOps<OT>, R: TypeRewriteRule<OT, O>> {
    field_name: String,
    rule: R,
    _phantom: PhantomData<(OT, O)>,
}

impl<OT: Clone, O: CodecOps<OT>, R: TypeRewriteRule<OT, O>> TypeRewriteRule<OT, O>
    for ApplyRuleToFieldRule<OT, O, R>
{
    fn fix_data(&self, ops: O, value: OT) -> OT {
        if ops.get_map(&value).is_ok() {
            let mut object = ops.take_map(value).unwrap();
            if let Ok(field_value) = object.take(&self.field_name) {
                object.set(
                    &self.field_name,
                    self.rule.fix_data(ops.clone(), field_value),
                );
            }
            return ops.create_map(object.entries());
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

pub struct RemoveFieldRule<OT: Clone, O: CodecOps<OT>> {
    field_name: String,
    _phantom: PhantomData<(OT, O)>,
}

impl<OT: Clone, O: CodecOps<OT>> TypeRewriteRule<OT, O> for RemoveFieldRule<OT, O> {
    fn fix_data(&self, ops: O, mut value: OT) -> OT {
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
