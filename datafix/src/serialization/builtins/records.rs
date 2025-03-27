use core::{cell::OnceCell, marker::PhantomData};

use crate::{
    result::{DataError, DataResult},
    serialization::{Codec, CodecOps, Context, MapView},
};
use alloc::string::String;

pub trait MapFieldGetter<T, C: Codec<T, O>, Struct, Rt, O: CodecOps> {
    fn encode_into(
        &self,
        ops: &O,
        value: &Struct,
        ctx: &mut Context,
    ) -> Option<DataResult<(String, O::T)>>;
    fn get_field(&self, ops: &O, value: &O::T, ctx: &mut Context) -> DataResult<Rt>;
    fn field_name(&self) -> &str;
}

pub struct OptionalField<T, C: Codec<T, O>, Struct, O: CodecOps> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &Option<T>,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> O>,
}

impl<T, C: Codec<T, O>, Struct, O: CodecOps> MapFieldGetter<T, C, Struct, Option<T>, O>
    for OptionalField<T, C, Struct, O>
{
    fn encode_into(
        &self,
        ops: &O,
        value: &Struct,
        ctx: &mut Context,
    ) -> Option<DataResult<(String, O::T)>> {
        let value = (self.getter)(value);
        match value {
            Some(value) => {
                let e = self.codec.encode(ops, value, ctx);
                let e = match e {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok((self.field_name.clone(), e)))
            }
            None => None,
        }
    }

    fn get_field(&self, ops: &O, value: &O::T, ctx: &mut Context) -> DataResult<Option<T>> {
        let obj = ops.get_map(value)?;
        match obj.get(&self.field_name) {
            Ok(field) => Ok(Some(self.codec.decode(ops, field, ctx)?)),
            Err(_) => Ok(None),
        }
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }
}

pub struct DefaultField<T, C: Codec<T, O>, Struct, O: CodecOps, F: Fn() -> T> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &T,
    pub(crate) codec: C,
    pub(crate) default: F,
    pub(crate) _phantom: PhantomData<fn() -> (O::T, O)>,
}

impl<T, C: Codec<T, O>, Struct, O: CodecOps, F: Fn() -> T> MapFieldGetter<T, C, Struct, T, O>
    for DefaultField<T, C, Struct, O, F>
{
    fn encode_into(
        &self,
        ops: &O,
        value: &Struct,
        ctx: &mut Context,
    ) -> Option<DataResult<(String, O::T)>> {
        let value = (self.getter)(value);
        let e = self.codec.encode(ops, value, ctx);
        let e = match e {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok((self.field_name.clone(), e)))
    }

    fn get_field(&self, ops: &O, value: &O::T, ctx: &mut Context) -> DataResult<T> {
        let obj = ops.get_map(value)?;
        match obj.get(&self.field_name) {
            Ok(field) => Ok(self.codec.decode(ops, field, ctx)?),
            Err(_) => {
                let default_value = (self.default)();
                Ok(default_value)
            }
        }
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }
}

pub struct RecordField<T, C: Codec<T, O>, Struct, O: CodecOps> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, O::T, O)>,
}

impl<T, C: Codec<T, O>, Struct, O: CodecOps> MapFieldGetter<T, C, Struct, T, O>
    for RecordField<T, C, Struct, O>
{
    fn get_field(&self, ops: &O, value: &O::T, ctx: &mut Context) -> DataResult<T> {
        let obj = ops.get_map(value)?;
        let field = obj.get(&self.field_name)?;
        self.codec.decode(ops, field, ctx)
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }

    fn encode_into(
        &self,
        ops: &O,
        value: &Struct,
        ctx: &mut Context,
    ) -> Option<DataResult<(String, O::T)>> {
        let e = self.codec.encode(ops, (self.getter)(value), ctx);
        let e = match e {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok((self.field_name.clone(), e)))
    }
}

pub struct UnitCodec {}

impl<O: CodecOps> Codec<(), O> for UnitCodec {
    fn encode(&self, ops: &O, _value: &(), _ctx: &mut Context) -> DataResult<O::T> {
        Ok(ops.create_unit())
    }

    fn decode(&self, ops: &O, value: &O::T, _ctx: &mut Context) -> DataResult<()> {
        ops.get_unit(value)
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
                $codec: Codec<$name, O>,
                $field_return_type,
                $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type, O>
            ),*,
            Struct, O: CodecOps
        > {
            $(pub(crate) $field: $field_type),*,
            pub(crate) into_struct: OnceCell<fn($($field_return_type),*) -> Struct>,
            pub(crate) _phantom: PhantomData<($($name, $codec, $field_return_type),*, O)>
        }

        #[doc(hidden)]
        impl<Struct, $(
            $name,
            $codec: Codec<$name, O>,
            $field_return_type,
            $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type, O>
        ),*, O: CodecOps> Codec<Struct, O> for $struct_name<$($name, $codec, $field_return_type, $field_type),*, Struct, O> {
            fn encode(&self, ops: &O, value: &Struct, ctx: &mut Context) -> DataResult<O::T> {
                ops.create_map_special([
                    $(self.$field.encode_into(ops, value, ctx),)*
                ])
            }

            fn decode(&self, ops: &O, value: &O::T, ctx: &mut Context) -> DataResult<Struct> {
                $(
                    ctx.push_field(self.$field.field_name());
                    let $field: $field_return_type = self.$field.get_field(ops, value, ctx)?;
                    ctx.pop();
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
