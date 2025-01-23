use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use crate::result::{DataError, DataResult};

use super::{Codec, ops::CodecOps};

pub struct RecordField<T, C: Codec<T>, S> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&S) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
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
        fields: { $($field:ident: $name:ident[$codec:ident]),* }
    ) => {
        pub struct $struct_name<$($name, $codec: Codec<$name>),*, Struct> {
            $(pub(crate) $field: RecordField<$name, $codec, Struct>),*,
            pub(crate) into_struct: OnceCell<fn($($name),*) -> Struct>
        }

        #[doc(hidden)]
        impl<$($name, $codec: Codec<$name>),*, Struct> Codec<Struct> for $struct_name<$($name, $codec),*, Struct> {
            fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
                Ok(ops.create_object(&[
                    $((
                        &self.$field.field_name,
                        self.$field.codec.encode(ops, (self.$field.getter)(value))?,
                    )),*
                ]))
            }

            fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Struct> {
                let obj = ops.get_object(value)?;
                $(
                    let Some($field) = obj.get(&self.$field.field_name) else {
                        return Err(DataError::new(&alloc::format!("No key \"{}\" in object", self.$field.field_name)));
                    };
                )*

                let slice = [$(&self.$field.field_name),*];
                for key in obj.keys() {
                    if !slice.contains(&key) {
                        return Err(DataError::new(&alloc::format!("Unsupported key \"{}\" in object", key)))
                    }
                }

                Ok((self.into_struct.get().unwrap())(
                    $((self.$field.codec.decode(ops, $field))?),*
                ))
            }
        }
    };
}

record_codec! {
    name: RecordCodec1,
    fields: {
        codec1: P1[P1C]
    }
}

record_codec! {
    name: RecordCodec2,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C]
    }
}

record_codec! {
    name: RecordCodec3,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C]
    }
}

record_codec! {
    name: RecordCodec4,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C]
    }
}

record_codec! {
    name: RecordCodec5,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C]
    }
}

record_codec! {
    name: RecordCodec6,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C]
    }
}

record_codec! {
    name: RecordCodec7,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C]
    }
}

record_codec! {
    name: RecordCodec8,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C]
    }
}

record_codec! {
    name: RecordCodec9,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C]
    }
}

record_codec! {
    name: RecordCodec10,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C]
    }
}

record_codec! {
    name: RecordCodec11,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C]
    }
}

record_codec! {
    name: RecordCodec12,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C],
        codec12: P12[P12C]
    }
}

record_codec! {
    name: RecordCodec13,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C],
        codec12: P12[P12C],
        codec13: P13[P13C]
    }
}

record_codec! {
    name: RecordCodec14,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C],
        codec12: P12[P12C],
        codec13: P13[P13C],
        codec14: P14[P14C]
    }
}

record_codec! {
    name: RecordCodec15,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C],
        codec12: P12[P12C],
        codec13: P13[P13C],
        codec14: P14[P14C],
        codec15: P15[P15C]
    }
}

record_codec! {
    name: RecordCodec16,
    fields: {
        codec1: P1[P1C],
        codec2: P2[P2C],
        codec3: P3[P3C],
        codec4: P4[P4C],
        codec5: P5[P5C],
        codec6: P6[P6C],
        codec7: P7[P7C],
        codec8: P8[P8C],
        codec9: P9[P9C],
        codec10: P10[P10C],
        codec11: P11[P11C],
        codec12: P12[P12C],
        codec13: P13[P13C],
        codec14: P14[P14C],
        codec15: P15[P15C],
        codec16: P16[P16C]
    }
}
