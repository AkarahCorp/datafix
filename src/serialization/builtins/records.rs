use core::{cell::OnceCell, marker::PhantomData};

use crate::{
    fixers::Type,
    result::{DataError, DataResult},
    serialization::{Codec, CodecOps, MapView},
};
use alloc::string::String;

pub trait MapFieldGetter<T, C: Codec<T>, Struct, Rt> {
    fn encode_into<U, O: CodecOps<U>>(
        &self,
        ops: &O,
        value: &Struct,
    ) -> Option<DataResult<(String, U)>>;
    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<Rt>;
    fn field_name(&self) -> &str;
    fn field_type(&self) -> Type;
}

pub struct OptionalField<T, C: Codec<T>, Struct> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &Option<T>,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T, C: Codec<T>, Struct> MapFieldGetter<T, C, Struct, Option<T>>
    for OptionalField<T, C, Struct>
{
    fn encode_into<U, O: CodecOps<U>>(
        &self,
        ops: &O,
        value: &Struct,
    ) -> Option<DataResult<(String, U)>> {
        let value = (self.getter)(value);
        match value {
            Some(value) => {
                let e = self.codec.encode(ops, value);
                let e = match e {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok((self.field_name.clone(), e)))
            }
            None => None,
        }
    }

    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<Option<T>> {
        let mut obj = ops.get_map(value)?;
        match obj.get(&self.field_name) {
            Ok(field) => Ok(Some(self.codec.decode(ops, field)?)),
            Err(e) => Err(e),
        }
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }

    fn field_type(&self) -> Type {
        self.codec.get_type()
    }
}

pub struct RecordField<T, C: Codec<T>, Struct> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T, C: Codec<T>, Struct> MapFieldGetter<T, C, Struct, T> for RecordField<T, C, Struct> {
    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<T> {
        let mut obj = ops.get_map(value)?;
        let field = obj.get(&self.field_name)?;
        self.codec.decode(ops, field)
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }

    fn encode_into<U, O: CodecOps<U>>(
        &self,
        ops: &O,
        value: &Struct,
    ) -> Option<DataResult<(String, U)>> {
        let e = self.codec.encode(ops, (self.getter)(value));
        let e = match e {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok((self.field_name.clone(), e)))
    }

    fn field_type(&self) -> Type {
        self.codec.get_type()
    }
}

pub struct UnitCodec {}

impl Codec<()> for UnitCodec {
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, _value: &()) -> DataResult<U> {
        Ok(ops.create_unit())
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<()> {
        ops.get_unit(value)
    }

    fn get_type(&self) -> crate::fixers::Type {
        Type::unit()
    }
}

/// I'm sorry. Not even God himself understands this macro anymore.
macro_rules! record_codec {
    (
        name: $struct_name:ident,
        fields: { $($field:ident: $name:ident[$codec:ident; $field_type:ident; $field_return_type:ident]),* }
    ) => {
        pub struct $struct_name<$(
                $name,
                $codec: Codec<$name>,
                $field_return_type,
                $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type>
            ),*,
            Struct
        > {
            $(pub(crate) $field: $field_type),*,
            pub(crate) into_struct: OnceCell<fn($($field_return_type),*) -> Struct>,
            pub(crate) _phantom: PhantomData<($($name, $codec, $field_return_type),*,)>
        }

        #[doc(hidden)]
        impl<Struct, $(
            $name,
            $codec: Codec<$name>,
            $field_return_type,
            $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type>
        ),*> Codec<Struct> for $struct_name<$($name, $codec, $field_return_type, $field_type),*, Struct> {
            fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
                ops.create_map_special([
                    $(self.$field.encode_into(ops, value),)*
                ])
            }

            fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &mut U) -> DataResult<Struct> {
                $(
                    let $field: $field_return_type = self.$field.get_field(ops, value)?;
                )*
                let map = ops.get_map(value)?;
                let slice = [$(&self.$field.field_name()),*];
                for key in map.keys() {
                    if !slice.contains(&&&*key) {
                        return Err(DataError::new_custom(&alloc::format!("Unsupported key \"{}\" in object", key)))
                    }
                }

                Ok((self.into_struct.get().unwrap())(
                    $($field),*
                ))
            }

            fn get_type(&self) -> crate::fixers::Type {
                let mut map = crate::fixers::TypeMap::new();
                $(
                    map.insert_field(self.$field.field_name(), self.$field.field_type());
                )*
                crate::fixers::Type::map(map)
            }
        }
    };
}

record_codec! {
    name: MapCodec1,
    fields: {
        codec1: P1[P1C; P1F; P1R]
    }
}

record_codec! {
    name: MapCodec2,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R]
    }
}

record_codec! {
    name: MapCodec3,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R]
    }
}

record_codec! {
    name: MapCodec4,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R]
    }
}

record_codec! {
    name: MapCodec5,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R]
    }
}

record_codec! {
    name: MapCodec6,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R]
    }
}

record_codec! {
    name: MapCodec7,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R]
    }
}

record_codec! {
    name: MapCodec8,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R]
    }
}

record_codec! {
    name: MapCodec9,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R]
    }
}

record_codec! {
    name: MapCodec10,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R]
    }
}

record_codec! {
    name: MapCodec11,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R]
    }
}

record_codec! {
    name: MapCodec12,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R],
        codec12: P12[P12C; P12F; P12R]
    }
}

record_codec! {
    name: MapCodec13,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R],
        codec12: P12[P12C; P12F; P12R],
        codec13: P13[P13C; P13F; P13R]
    }
}

record_codec! {
    name: MapCodec14,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R],
        codec12: P12[P12C; P12F; P12R],
        codec13: P13[P13C; P13F; P13R],
        codec14: P14[P14C; P14F; P14R]
    }
}

record_codec! {
    name: MapCodec15,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R],
        codec12: P12[P12C; P12F; P12R],
        codec13: P13[P13C; P13F; P13R],
        codec14: P14[P14C; P14F; P14R],
        codec15: P15[P15C; P15F; P15R]
    }
}

record_codec! {
    name: MapCodec16,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R],
        codec6: P6[P6C; P6F; P6R],
        codec7: P7[P7C; P7F; P7R],
        codec8: P8[P8C; P8F; P8R],
        codec9: P9[P9C; P9F; P9R],
        codec10: P10[P10C; P10F; P10R],
        codec11: P11[P11C; P11F; P11R],
        codec12: P12[P12C; P12F; P12R],
        codec13: P13[P13C; P13F; P13R],
        codec14: P14[P14C; P14F; P14R],
        codec15: P15[P15C; P15F; P15R],
        codec16: P16[P16C; P16F; P16R]
    }
}
