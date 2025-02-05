use core::{cell::OnceCell, marker::PhantomData};

use crate::{builtins::records::*, serialization::Codec};

pub struct RecordCodecBuilder<InnerCodec> {
    pub(crate) codec: InnerCodec,
}

impl RecordCodecBuilder<UnitCodec> {
    pub fn new() -> RecordCodecBuilder<UnitCodec> {
        RecordCodecBuilder {
            codec: UnitCodec {},
        }
    }

    pub fn field<
        P1,
        P1C: Codec<P1>,
        P1R,
        NxtField: RecordFieldGetter<P1, P1C, Struct, P1R>,
        Struct,
    >(
        self,
        field: NxtField,
    ) -> RecordCodecBuilder<RecordCodec1<P1, P1C, P1R, NxtField, Struct>> {
        RecordCodecBuilder {
            codec: RecordCodec1 {
                codec1: field,
                into_struct: OnceCell::new(),
                _phantom: PhantomData,
            },
        }
    }
}

/// I'm sorry. Not even God himself understands this macro anymore.
macro_rules! impl_record_codec_builder {
    (
        type: $type:ident,
        fields: { $(
            $field:ident: $name:ident[$codec:ident; $field_type:ident; $field_return_type:ident]
        ),* },
        next:
        $next_field_name:ident:
        $next_type:ident as
        $next_name:ident[$next_codec:ident; $next_field_type:ident; $next_field_return_type:ident]
    ) => {
        #[doc(hidden)]
        impl<
            $(
                $name,
                $codec: Codec<$name>,
                $field_return_type,
                $field_type: RecordFieldGetter<$name, $codec, Struct, $field_return_type>
            ),*
            , Struct
        > RecordCodecBuilder<
            $type<
                $(
                    $name,
                    $codec,
                    $field_return_type,
                    $field_type
                ),*
                , Struct
            >
        > {
            pub fn field<
                $next_name,
                $next_codec: Codec<$next_name>,
                $next_field_return_type,
                NxtField: RecordFieldGetter<
                    $next_name,
                    $next_codec,
                    Struct,
                    $next_field_return_type>>
            (
                self,
                field: NxtField
            ) -> RecordCodecBuilder<
                $next_type<
                    $($name, $codec, $field_return_type, $field_type),*,
                    $next_name, $next_codec, $next_field_return_type, NxtField, Struct
                >
            > {
                RecordCodecBuilder {
                    codec: $next_type {
                        $($field: self.codec.$field),*,
                        $next_field_name: field,
                        into_struct: OnceCell::new(),
                        _phantom: PhantomData,
                    },
                }
            }

            pub fn build(self, into_struct: fn($($field_return_type),*) -> Struct) -> impl Codec<Struct> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

macro_rules! impl_record_codec_builder_last {
    (
        type: $type:ident,
        fields: { $($field:ident: $name:ident[$codec:ident; $field_type:ident; $field_return_type:ident]),* }
    ) => {
        impl<$($name, $codec: Codec<$name>, $field_return_type, $field_type: RecordFieldGetter<$name, $codec, Struct, $field_return_type>),*, Struct>
            RecordCodecBuilder<$type<$($name, $codec, $field_return_type, $field_type),*, Struct>> {
            pub fn build(self, into_struct: fn($($field_return_type),*) -> Struct) -> impl Codec<Struct> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

impl_record_codec_builder! {
    type: RecordCodec1,
    fields: { codec1: P1[P1C; P1F; P1R] },
    next: codec2: RecordCodec2 as P2[P2C; P2F; P2R]
}

impl_record_codec_builder! {
    type: RecordCodec2,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R] },
    next: codec3: RecordCodec3 as P3[P3C; P3F; P3R]
}
impl_record_codec_builder! {
    type: RecordCodec3,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R] },
    next: codec4: RecordCodec4 as P4[P4C; P4F; P4R]
}

impl_record_codec_builder! {
    type: RecordCodec4,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R] },
    next: codec5: RecordCodec5 as P5[P5C; P5F; P5R]
}

impl_record_codec_builder! {
    type: RecordCodec5,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R] },
    next: codec6: RecordCodec6 as P6[P6C; P6F; P6R]
}

impl_record_codec_builder! {
    type: RecordCodec6,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R] },
    next: codec7: RecordCodec7 as P7[P7C; P7F; P7R]
}

impl_record_codec_builder! {
    type: RecordCodec7,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R] },
    next: codec8: RecordCodec8 as P8[P8C; P8F; P8R]
}

impl_record_codec_builder! {
    type: RecordCodec8,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R] },
    next: codec9: RecordCodec9 as P9[P9C; P9F; P9R]
}

impl_record_codec_builder! {
    type: RecordCodec9,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R], codec9: P9[P9C; P9F; P9R] },
    next: codec10: RecordCodec10 as P10[P10C; P10F; P10R]
}

impl_record_codec_builder_last! {
    type: RecordCodec10,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R], codec9: P9[P9C; P9F; P9R], codec10: P10[P10C; P10F; P10R] }
}
