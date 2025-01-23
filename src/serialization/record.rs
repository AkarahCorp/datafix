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

#[cfg(test)]
mod tests {
    use crate::{
        dynamic::Dynamic,
        serialization::{Codec, DefaultCodec, RecordCodecBuilder},
    };

    #[derive(PartialEq, Debug)]
    pub struct Pos2d {
        x: f64,
        y: f64,
    }

    impl Pos2d {
        pub fn new(x: f64, y: f64) -> Pos2d {
            Pos2d { x, y }
        }

        pub fn x(&self) -> &f64 {
            &self.x
        }

        pub fn y(&self) -> &f64 {
            &self.y
        }

        pub fn codec() -> impl Codec<Pos2d> {
            RecordCodecBuilder::new()
                .field(f64::codec().field_of("x", Pos2d::x))
                .field(f64::codec().field_of("y", Pos2d::y))
                .build(Pos2d::new)
        }
    }

    #[test]
    fn struct_codec() {
        let value = Pos2d::new(15.0, 30.0);

        let encoded = Pos2d::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = Pos2d::codec().decode(&Dynamic::ops(), &encoded).unwrap();

        assert_eq!(value, decoded);
    }
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

record_codec! {
    name: RecordCodec3,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F]
    }
}

record_codec! {
    name: RecordCodec4,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F]
    }
}

record_codec! {
    name: RecordCodec5,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F]
    }
}

record_codec! {
    name: RecordCodec6,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F]
    }
}

record_codec! {
    name: RecordCodec7,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F]
    }
}

record_codec! {
    name: RecordCodec8,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F]
    }
}

record_codec! {
    name: RecordCodec9,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F]
    }
}

record_codec! {
    name: RecordCodec10,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F]
    }
}

record_codec! {
    name: RecordCodec11,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F]
    }
}

record_codec! {
    name: RecordCodec12,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F],
        codec12: P12[P12C; P12F]
    }
}

record_codec! {
    name: RecordCodec13,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F],
        codec12: P12[P12C; P12F],
        codec13: P13[P13C; P13F]
    }
}

record_codec! {
    name: RecordCodec14,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F],
        codec12: P12[P12C; P12F],
        codec13: P13[P13C; P13F],
        codec14: P14[P14C; P14F]
    }
}

record_codec! {
    name: RecordCodec15,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F],
        codec12: P12[P12C; P12F],
        codec13: P13[P13C; P13F],
        codec14: P14[P14C; P14F],
        codec15: P15[P15C; P15F]
    }
}

record_codec! {
    name: RecordCodec16,
    fields: {
        codec1: P1[P1C; P1F],
        codec2: P2[P2C; P2F],
        codec3: P3[P3C; P3F],
        codec4: P4[P4C; P4F],
        codec5: P5[P5C; P5F],
        codec6: P6[P6C; P6F],
        codec7: P7[P7C; P7F],
        codec8: P8[P8C; P8F],
        codec9: P9[P9C; P9F],
        codec10: P10[P10C; P10F],
        codec11: P11[P11C; P11F],
        codec12: P12[P12C; P12F],
        codec13: P13[P13C; P13F],
        codec14: P14[P14C; P14F],
        codec15: P15[P15C; P15F],
        codec16: P16[P16C; P16F]
    }
}
