use core::marker::PhantomData;

use crate::serialization::CodecOps;

use super::{TyVal, Type, TypeRewriteRule};

pub struct Rules;

impl Rules {
    pub fn map<OT, O: CodecOps<OT>, F: Fn(TyVal<OT, O>) -> TyVal<OT, O>>(
        function: F,
    ) -> impl TypeRewriteRule<OT, O> {
        MapFieldRule {
            function,
            _phantom: PhantomData,
        }
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
