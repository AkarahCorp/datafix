use core::marker::PhantomData;

use crate::{
    result::{DataError, DataResult},
    serialization::{CodecOps, MapViewMut, OwnedMapView},
};

use super::{Type, TypeRewriteRule};

pub enum TyVal<OT, O: CodecOps<OT>> {
    Value(OT, O),
    Type(Type),
}

impl<OT: Clone, O: CodecOps<OT>> Clone for TyVal<OT, O> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(arg0, arg1) => Self::Value(arg0.clone(), arg1.clone()),
            Self::Type(arg0) => Self::Type(arg0.clone()),
        }
    }
}

impl<OT, O: CodecOps<OT>> TyVal<OT, O> {
    pub fn map<F: Fn(TyVal<OT, O>) -> TyVal<OT, O>>(function: F) -> impl TypeRewriteRule<OT, O> {
        MapFieldRule {
            function,
            _phantom: PhantomData,
        }
    }

    pub fn take(self, field: &str) -> DataResult<TyVal<OT, O>> {
        match self {
            TyVal::Value(value, ops) => ops
                .take_map(value)
                .and_then(|x| x.take(field))
                .map(|x| TyVal::Value(x, ops.clone())),
            TyVal::Type(ty) => match ty {
                Type::Object(object_type) => object_type.get(field).map(|x| TyVal::Type(x)),
                _ => Err(DataError::unexpected_type("object")),
            },
        }
    }

    pub fn value(self) -> DataResult<OT> {
        match self {
            TyVal::Value(value, ..) => Ok(value),
            TyVal::Type(..) => Err(DataError::new_custom("not a value")),
        }
    }

    pub fn ty(self) -> DataResult<Type> {
        match self {
            TyVal::Value(..) => Err(DataError::new_custom("not a type")),
            TyVal::Type(ty) => Ok(ty),
        }
    }

    pub fn with(mut self, field: &str, insert_value: TyVal<OT, O>) -> TyVal<OT, O> {
        match &mut self {
            TyVal::Value(value, ops) => {
                let mut obj = ops.get_map_mut(value);
                if let Ok(obj) = &mut obj {
                    if let Ok(insert_value) = insert_value.value() {
                        obj.set(field, insert_value);
                    }
                }
            }
            TyVal::Type(ty) => {
                if let Type::Object(object) = ty {
                    if let Ok(ty) = insert_value.ty() {
                        object.insert(field, ty);
                    }
                }
            }
        }
        self
    }
}

pub struct MapFieldRule<OT, O: CodecOps<OT>, F: Fn(TyVal<OT, O>) -> TyVal<OT, O>> {
    function: F,
    _phantom: PhantomData<(OT, O)>,
}

impl<OT, O: CodecOps<OT>, F: Fn(TyVal<OT, O>) -> TyVal<OT, O>> TypeRewriteRule<OT, O>
    for MapFieldRule<OT, O, F>
{
    fn fix_data(&self, ops: O, value: OT) -> OT {
        let result = (self.function)(TyVal::Value(value, ops));
        match result {
            TyVal::Value(value, _) => value,
            TyVal::Type(_) => panic!(),
        }
    }

    fn fix_type(&self, ty: Type) -> Type {
        let result = (self.function)(TyVal::Type(ty));
        match result {
            TyVal::Value(_, _) => panic!(),
            TyVal::Type(ty) => ty,
        }
    }
}
