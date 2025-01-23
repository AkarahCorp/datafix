use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use super::{Codec, record::*};

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
        fields: { $($field:ident: $name:ident[$codec:ident]),* }
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

impl_record_codec_builder! {
    type: RecordCodec6,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C] },
    next: codec7: RecordCodec7 as P7[P7C]
}

impl_record_codec_builder! {
    type: RecordCodec7,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C] },
    next: codec8: RecordCodec8 as P8[P8C]
}

impl_record_codec_builder! {
    type: RecordCodec8,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C] },
    next: codec9: RecordCodec9 as P9[P9C]
}

impl_record_codec_builder! {
    type: RecordCodec9,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C] },
    next: codec10: RecordCodec10 as P10[P10C]
}

impl_record_codec_builder! {
    type: RecordCodec10,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C] },
    next: codec11: RecordCodec11 as P11[P11C]
}

impl_record_codec_builder! {
    type: RecordCodec11,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C], codec11: P11[P11C] },
    next: codec12: RecordCodec12 as P12[P12C]
}

impl_record_codec_builder! {
    type: RecordCodec12,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C], codec11: P11[P11C], codec12: P12[P12C] },
    next: codec13: RecordCodec13 as P13[P13C]
}

impl_record_codec_builder! {
    type: RecordCodec13,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C], codec11: P11[P11C], codec12: P12[P12C], codec13: P13[P13C] },
    next: codec14: RecordCodec14 as P14[P14C]
}

impl_record_codec_builder! {
    type: RecordCodec14,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C], codec11: P11[P11C], codec12: P12[P12C], codec13: P13[P13C], codec14: P14[P14C] },
    next: codec15: RecordCodec15 as P15[P15C]
}

impl_record_codec_builder! {
    type: RecordCodec15,
    fields: { codec1: P1[P1C], codec2: P2[P2C], codec3: P3[P3C], codec4: P4[P4C], codec5: P5[P5C], codec6: P6[P6C], codec7: P7[P7C], codec8: P8[P8C], codec9: P9[P9C], codec10: P10[P10C], codec11: P11[P11C], codec12: P12[P12C], codec13: P13[P13C], codec14: P14[P14C], codec15: P15[P15C] },
    next: codec16: RecordCodec16 as P16[P16C]
}

impl_record_codec_builder_last! {
    type: RecordCodec16,
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

    #[derive(Debug, PartialEq)]
    struct VeryBigRecord {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
    }

    impl VeryBigRecord {
        pub fn codec() -> impl Codec<VeryBigRecord> {
            RecordCodecBuilder::new()
                .field(f64::codec(), "a", |x: &VeryBigRecord| &x.a)
                .field(f64::codec(), "b", |x: &VeryBigRecord| &x.b)
                .field(f64::codec(), "c", |x: &VeryBigRecord| &x.c)
                .field(f64::codec(), "d", |x: &VeryBigRecord| &x.d)
                .field(f64::codec(), "e", |x: &VeryBigRecord| &x.e)
                .field(f64::codec(), "f", |x: &VeryBigRecord| &x.f)
                .field(f64::codec(), "g", |x: &VeryBigRecord| &x.g)
                .build(|a, b, c, d, e, f, g| VeryBigRecord {
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    g,
                })
        }
    }

    #[test]
    fn very_big_record() {
        let data = VeryBigRecord {
            a: 0.0,
            b: 5.7,
            c: 56.3,
            d: -56.2,
            e: 0.1,
            f: 0.001,
            g: 13.5,
        };
        let encoded = VeryBigRecord::codec()
            .encode(&Dynamic::ops(), &data)
            .unwrap();
        let decoded = VeryBigRecord::codec()
            .decode(&Dynamic::ops(), &encoded)
            .unwrap();
        assert_eq!(data, decoded);
    }
}
