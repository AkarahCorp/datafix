use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use crate::result::{DataError, DataResult};

use super::{Codec, ops::CodecOps};

pub trait RecordFieldGetter<T, C: Codec<T>, Struct> {
    fn encode_into<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<(&str, U)>;
    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<T>;
    fn field_name(&self) -> &str;
}

pub struct RecordField<T, C: Codec<T>, Struct> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T, C: Codec<T>, Struct> RecordFieldGetter<T, C, Struct> for RecordField<T, C, Struct> {
    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<T> {
        let obj = ops.get_object(&value)?;
        let field = obj.get(&self.field_name).ok_or_else(|| {
            DataError::new(&alloc::format!(
                "Expected key \"{}\" in object",
                self.field_name
            ))
        })?;
        self.codec.decode(ops, field)
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }

    fn encode_into<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<(&str, U)> {
        Ok((
            &self.field_name,
            self.codec.encode(ops, (self.getter)(value))?,
        ))
    }
}

pub struct UnitCodec {}

impl Codec<()> for UnitCodec {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, _value: &()) -> DataResult<U> {
        Ok(ops.create_unit())
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<()> {
        ops.get_unit(value)
    }
}

macro_rules! record_codec {
    (
        name: $struct_name:ident,
        fields: { $($field:ident: $name:ident[$codec:ident; $field_type:ident]),* }
    ) => {
        pub struct $struct_name<$($name, $codec: Codec<$name>, $field_type: RecordFieldGetter<$name, $codec, Struct>),*, Struct> {
            $(pub(crate) $field: $field_type),*,
            pub(crate) into_struct: OnceCell<fn($($name),*) -> Struct>,
            pub(crate) _phantom: PhantomData<($($codec),*,)>
        }

        #[doc(hidden)]
        impl<$(
            $name,
            $codec: Codec<$name>,
            $field_type: RecordFieldGetter<$name, $codec, Struct>
        ),*, Struct> Codec<Struct> for $struct_name<$($name, $codec, $field_type),*, Struct> {
            fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
                Ok(ops.create_object(&[
                    $(self.$field.encode_into(ops, value)?,)*
                ]))
            }

            fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Struct> {
                let obj = ops.get_object(value)?;
                $(
                    let $field: $name = self.$field.get_field(ops, &value)?;
                    // let Some($field) = obj.get(&self.$field.field_name) else {
                    //     return Err(DataError::new(&alloc::format!("No) key \"{}\" in object", self.$field.field_name)));
                    // };
                )*

                let slice = [$(&self.$field.field_name()),*];
                for key in obj.keys() {
                    if !slice.contains(&&key.as_str()) {
                        return Err(DataError::new(&alloc::format!("Unsupported key \"{}\" in object", key)))
                    }
                }

                Ok((self.into_struct.get().unwrap())(
                    // $((self.$field.codec.decode(ops, $field))?),*
                    $($field),*
                ))
            }
        }
    };
}

record_codec! {
    name: RecordCodec1,
    fields: {
        codec1: P1[P1C; P1F]
    }
}

record_codec! {
    name: RecordCodec2,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F]
    }
}
