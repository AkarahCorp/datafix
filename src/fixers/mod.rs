mod dynamic;
pub use dynamic::*;
mod schema;
pub use schema::*;
mod types;
pub use types::*;
pub mod builtins;

use crate::serialization::CodecOps;

pub trait Fixer {
    fn fix_data<T, O: CodecOps<T>>(&self, data: Dynamic<'_, T, O>, ops: O);
    fn fix_type(&self, type_name: &TypeReference, input: &mut Type);
}
