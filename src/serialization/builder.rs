use core::{cell::OnceCell, marker::PhantomData};

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

    pub fn field<P1, P1C: Codec<P1>, NxtField: RecordFieldGetter<P1, P1C, Struct>, Struct>(
        self,
        field: NxtField,
    ) -> RecordCodecBuilder<RecordCodec1<P1, P1C, NxtField, Struct>> {
        RecordCodecBuilder {
            codec: RecordCodec1 {
                codec1: field,
                into_struct: OnceCell::new(),
                _phantom: PhantomData,
            },
        }
    }
}

macro_rules! impl_record_codec_builder {
    (
        type: $type:ident,
        fields: { $($field:ident: $name:ident[$codec:ident; $field_type:ident]),* },
        next: $next_field_name:ident: $next_type:ident as $next_name:ident[$next_codec:ident; $next_field_type:ident]
    ) => {
        impl<$($name, $codec: Codec<$name>, $field_type: RecordFieldGetter<$name, $codec, Struct>),*, Struct> RecordCodecBuilder<$type<$($name, $codec, $field_type),*, Struct>> {
            pub fn field<$next_name, $next_codec: Codec<$next_name>, NxtField: RecordFieldGetter<$next_name, $next_codec, Struct>>(
                self,
                field: NxtField
            ) -> RecordCodecBuilder<$next_type<$($name, $codec, $field_type),*, $next_name, $next_codec, NxtField, Struct>> {
                RecordCodecBuilder {
                    codec: $next_type {
                        $($field: self.codec.$field),*,
                        $next_field_name: field,
                        into_struct: OnceCell::new(),
                        _phantom: PhantomData,
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
        fields: { $($field:ident: $name:ident[$codec:ident; $field_type:ident]),* }
    ) => {
        impl<$($name, $codec: Codec<$name>, $field_type: RecordFieldGetter<$name, $codec, Struct>),*, Struct> RecordCodecBuilder<$type<$($name, $codec, $field_type),*, Struct>> {
            pub fn build(self, into_struct: fn($($name),*) -> Struct) -> impl Codec<Struct> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

impl_record_codec_builder! {
    type: RecordCodec1,
    fields: { codec1: P1[P1C; P1F] },
    next: codec2: RecordCodec2 as P2[P2C; P2F]
}

impl_record_codec_builder! {
    type: RecordCodec2,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F] },
    next: codec3: RecordCodec3 as P3[P3C; P3F]
}

impl_record_codec_builder! {
    type: RecordCodec3,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F] },
    next: codec4: RecordCodec4 as P4[P4C; P4F]
}

impl_record_codec_builder! {
    type: RecordCodec4,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F] },
    next: codec5: RecordCodec5 as P5[P5C; P5F]
}

impl_record_codec_builder! {
    type: RecordCodec5,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F] },
    next: codec6: RecordCodec6 as P6[P6C; P6F]
}

impl_record_codec_builder! {
    type: RecordCodec6,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F] },
    next: codec7: RecordCodec7 as P7[P7C; P7F]
}

impl_record_codec_builder! {
    type: RecordCodec7,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F] },
    next: codec8: RecordCodec8 as P8[P8C; P8F]
}

impl_record_codec_builder! {
    type: RecordCodec8,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F] },
    next: codec9: RecordCodec9 as P9[P9C; P9F]
}

impl_record_codec_builder! {
    type: RecordCodec9,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F] },
    next: codec10: RecordCodec10 as P10[P10C; P10F]
}

impl_record_codec_builder! {
    type: RecordCodec10,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F] },
    next: codec11: RecordCodec11 as P11[P11C; P11F]
}

impl_record_codec_builder! {
    type: RecordCodec11,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F] },
    next: codec12: RecordCodec12 as P12[P12C; P12F]
}

impl_record_codec_builder! {
    type: RecordCodec12,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F], codec12: P12[P12C; P12F] },
    next: codec13: RecordCodec13 as P13[P13C; P13F]
}

impl_record_codec_builder! {
    type: RecordCodec13,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F], codec12: P12[P12C; P12F], codec13: P13[P13C; P13F] },
    next: codec14: RecordCodec14 as P14[P14C; P14F]
}

impl_record_codec_builder! {
    type: RecordCodec14,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F], codec12: P12[P12C; P12F], codec13: P13[P13C; P13F], codec14: P14[P14C; P14F] },
    next: codec15: RecordCodec15 as P15[P15C; P15F]
}

impl_record_codec_builder! {
    type: RecordCodec15,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F], codec12: P12[P12C; P12F], codec13: P13[P13C; P13F], codec14: P14[P14C; P14F], codec15: P15[P15C; P15F] },
    next: codec16: RecordCodec16 as P16[P16C; P16F]
}

impl_record_codec_builder_last! {
    type: RecordCodec16,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F], codec3: P3[P3C; P3F], codec4: P4[P4C; P4F], codec5: P5[P5C; P5F], codec6: P6[P6C; P6F], codec7: P7[P7C; P7F], codec8: P8[P8C; P8F], codec9: P9[P9C; P9F], codec10: P10[P10C; P10F], codec11: P11[P11C; P11F], codec12: P12[P12C; P12F], codec13: P13[P13C; P13F], codec14: P14[P14C; P14F], codec15: P15[P15C; P15F], codec16: P16[P16C; P16F] }
}
