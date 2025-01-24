use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use crate::result::{DataError, DataResult};

use super::{Codec, ops::CodecOps};

pub trait RecordFieldGetter<T, C: Codec<T>, Struct, Rt> {
    fn encode_into<U, O: CodecOps<U>>(
        &self,
        ops: &O,
        value: &Struct,
    ) -> Option<DataResult<(String, U)>>;
    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Rt>;
    fn field_name(&self) -> &str;
}

pub struct OptionalField<T, C: Codec<T>, Struct> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &Option<T>,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T, C: Codec<T>, Struct> RecordFieldGetter<T, C, Struct, Option<T>>
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

    fn get_field<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Option<T>> {
        let obj = ops.get_object(&value)?;
        match obj.get(&self.field_name) {
            Some(field) => Ok(Some(self.codec.decode(ops, field)?)),
            None => Ok(None),
        }
    }

    fn field_name(&self) -> &str {
        &self.field_name
    }
}

pub struct RecordField<T, C: Codec<T>, Struct> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&Struct) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

impl<T, C: Codec<T>, Struct> RecordFieldGetter<T, C, Struct, T> for RecordField<T, C, Struct> {
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
                $field_type: RecordFieldGetter<$name, $codec, Struct, $field_return_type>
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
            $field_type: RecordFieldGetter<$name, $codec, Struct, $field_return_type>
        ),*> Codec<Struct> for $struct_name<$($name, $codec, $field_return_type, $field_type),*, Struct> {
            fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
                ops.create_object_special([
                    $(self.$field.encode_into(ops, value),)*
                ])
            }

            fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Struct> {
                let obj = ops.get_object(value)?;
                $(
                    let $field: $field_return_type = self.$field.get_field(ops, &value)?;
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
    use alloc::string::{String, ToString};

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

    #[derive(PartialEq, Debug)]
    struct OptionalString {
        inner: Option<String>,
    }

    impl OptionalString {
        pub fn new(s: Option<String>) -> OptionalString {
            OptionalString { inner: s }
        }

        pub fn string(&self) -> &Option<String> {
            &self.inner
        }

        pub fn codec() -> impl Codec<OptionalString> {
            RecordCodecBuilder::new()
                .field(String::codec().optional_field_of("inner", OptionalString::string))
                .build(OptionalString::new)
        }
    }

    #[test]
    fn struct_optional_field_none() {
        let value = OptionalString { inner: None };

        let encoded = OptionalString::codec()
            .encode(&Dynamic::ops(), &value)
            .unwrap();
        let decoded = OptionalString::codec()
            .decode(&Dynamic::ops(), &encoded)
            .unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn struct_optional_field_present() {
        let value = OptionalString {
            inner: Some("Hello world!".to_string()),
        };

        let encoded = OptionalString::codec()
            .encode(&Dynamic::ops(), &value)
            .unwrap();
        let decoded = OptionalString::codec()
            .decode(&Dynamic::ops(), &encoded)
            .unwrap();

        assert_eq!(value, decoded);
    }
}

record_codec! {
    name: RecordCodec1,
    fields: {
        codec1: P1[P1C; P1F; P1R]
    }
}

record_codec! {
    name: RecordCodec2,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R]
    }
}

record_codec! {
    name: RecordCodec3,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R]
    }
}

record_codec! {
    name: RecordCodec4,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R]
    }
}

record_codec! {
    name: RecordCodec5,
    fields: {
        codec1: P1[P1C; P1F; P1R],
        codec2: P2[P2C; P2F; P2R],
        codec3: P3[P3C; P3F; P3R],
        codec4: P4[P4C; P4F; P4R],
        codec5: P5[P5C; P5F; P5R]
    }
}

record_codec! {
    name: RecordCodec6,
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
    name: RecordCodec7,
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
    name: RecordCodec8,
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
    name: RecordCodec9,
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
    name: RecordCodec10,
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
    name: RecordCodec11,
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
    name: RecordCodec12,
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
    name: RecordCodec13,
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
    name: RecordCodec14,
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
    name: RecordCodec15,
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
    name: RecordCodec16,
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
