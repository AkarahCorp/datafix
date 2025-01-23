use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use super::{
    Codec,
    record::{
        RecordCodec1, RecordCodec2, RecordCodec3, RecordCodec4, RecordCodec5, RecordCodec6,
        RecordField, UnitCodec,
    },
};

pub struct RecordCodecBuilder<InnerCodec> {
    pub(crate) codec: InnerCodec,
}

impl RecordCodecBuilder<UnitCodec> {
    pub fn new() -> RecordCodecBuilder<UnitCodec> {
        RecordCodecBuilder {
            codec: UnitCodec {},
        }
    }

    pub fn field<P1, P1C: Codec<P1>, Struct>(
        self,
        codec: P1C,
        field_name: impl Into<String>,
        getter: fn(&Struct) -> &P1,
    ) -> RecordCodecBuilder<RecordCodec1<P1, P1C, Struct>> {
        RecordCodecBuilder {
            codec: RecordCodec1 {
                codec1: RecordField {
                    field_name: field_name.into(),
                    getter,
                    codec,
                    _phantom: PhantomData,
                },
                into_struct: OnceCell::new(),
            },
        }
    }
}

macro_rules! impl_record_codec_builder {
    (
        type: $type:ident,
        fields: { $($field:ident: $name:ident[$codec:ident]),* },
        next: $next_field_name:ident: $next_type:ident as $next_name:ident[$next_codec:ident]
    ) => {
        impl<$($name, $codec: Codec<$name>),*, Struct> RecordCodecBuilder<$type<$($name, $codec),*, Struct>> {
            pub fn field<$next_name, $next_codec: Codec<$next_name>>(
                self,
                codec: $next_codec,
                field_name: impl Into<String>,
                getter: fn(&Struct) -> &$next_name,
            ) -> RecordCodecBuilder<$next_type<$($name, $codec),*, $next_name, $next_codec, Struct>> {
                RecordCodecBuilder {
                    codec: $next_type {
                        $($field: self.codec.$field),*,
                        $next_field_name: RecordField {
                            field_name: field_name.into(),
                            getter,
                            codec,
                            _phantom: PhantomData,
                        },
                        into_struct: OnceCell::new(),
                    },
                }
            }

            pub fn build(self, into_struct: fn($($name),*) -> Struct) -> impl Codec<Struct> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

macro_rules! impl_record_codec_builder_last {
    (
        type: $type:ident,
        fields: { $($field:ident: $name:ident[$codec:ident]),* },
    ) => {
        impl<$($name, $codec: Codec<$name>),*, Struct> RecordCodecBuilder<$type<$($name, $codec),*, Struct>> {
            pub fn build(self, into_struct: fn($($name),*) -> Struct) -> impl Codec<Struct> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

impl_record_codec_builder! {
    type: RecordCodec1,
    fields: { codec1: P1[P1C] },
    next: codec2: RecordCodec2 as P2[P2C]
}

impl_record_codec_builder! {
    type: RecordCodec2,
    fields: { codec1: P1[P1C], codec2: P2[P2C] },
    next: codec3: RecordCodec3 as P3[P3C]
}

impl_record_codec_builder! {
    type: RecordCodec3,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C] },
    next: codec4: RecordCodec4 as P4[P4C]
}

impl_record_codec_builder! {
    type: RecordCodec4,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C] },
    next: codec5: RecordCodec5 as P5[P5C]
}

impl_record_codec_builder! {
    type: RecordCodec5,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C] },
    next: codec6: RecordCodec6 as P6[P6C]
}

impl_record_codec_builder_last! {
    type: RecordCodec6,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C] },
}

#[cfg(test)]
mod tests {
    use crate::{
        dynamic::Dynamic,
        serialization::{Codec, DefaultCodec, builder::RecordCodecBuilder},
    };

    #[derive(Debug, PartialEq)]
    struct Pos4d {
        x: f64,
        y: f64,
        z: f64,
        w: f64,
    }

    impl Pos4d {
        fn new(x: f64, y: f64, z: f64, w: f64) -> Pos4d {
            Pos4d { x, y, z, w }
        }

        fn x(&self) -> &f64 {
            &self.x
        }

        fn y(&self) -> &f64 {
            &self.y
        }

        fn z(&self) -> &f64 {
            &self.z
        }

        fn w(&self) -> &f64 {
            &self.w
        }

        fn codec() -> impl Codec<Pos4d> {
            RecordCodecBuilder::new()
                .field(f64::codec(), "x", Pos4d::x)
                .field(f64::codec(), "y", Pos4d::y)
                .field(f64::codec(), "z", Pos4d::z)
                .field(f64::codec(), "w", Pos4d::w)
                .build(Pos4d::new)
        }
    }

    #[test]
    pub fn simple_record() {
        let value = Pos4d {
            x: 10.0,
            y: 15.0,
            z: 20.0,
            w: 30.0,
        };
        let encoded = Pos4d::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = Pos4d::codec().decode(&Dynamic::ops(), &encoded).unwrap();

        assert_eq!(decoded, value);
    }

    #[derive(PartialEq, Debug)]
    struct TopLevel {
        nested: Nested,
    }

    impl TopLevel {
        pub fn nested(&self) -> &Nested {
            &self.nested
        }
        pub fn new(nested: Nested) -> TopLevel {
            TopLevel { nested }
        }
        pub fn codec() -> impl Codec<TopLevel> {
            RecordCodecBuilder::new()
                .field(Nested::codec(), "nested", TopLevel::nested)
                .build(TopLevel::new)
        }
    }

    #[derive(PartialEq, Debug)]
    struct Nested {
        value: f64,
    }

    impl Nested {
        pub fn value(&self) -> &f64 {
            &self.value
        }
        pub fn new(value: f64) -> Nested {
            Nested { value }
        }
        pub fn codec() -> impl Codec<Nested> {
            RecordCodecBuilder::new()
                .field(f64::codec(), "value", Nested::value)
                .build(Nested::new)
        }
    }

    #[test]
    pub fn nested_structs() {
        let value = TopLevel {
            nested: Nested { value: 10.0 },
        };
        let encoded = TopLevel::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = TopLevel::codec().decode(&Dynamic::ops(), &encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
