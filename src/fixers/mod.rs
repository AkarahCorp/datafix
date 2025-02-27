mod types;
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
}
