use core::{cell::OnceCell, marker::PhantomData};

use crate::{
    serialization::builtins::records::*,
    serialization::{Codec, CodecOps},
};

use super::records::UnitCodec;

pub struct MapCodecBuilder<C, OT: Clone, O: CodecOps<OT>> {
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> (OT, O)>,
}

#[doc(hidden)]
impl<OT: Clone, O: CodecOps<OT>> Default for MapCodecBuilder<UnitCodec, OT, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<OT: Clone, O: CodecOps<OT>> MapCodecBuilder<UnitCodec, OT, O> {
    pub fn new() -> MapCodecBuilder<UnitCodec, OT, O> {
        MapCodecBuilder {
            codec: UnitCodec {},
            _phantom: PhantomData,
        }
    }

    pub fn field<
        P1,
        P1C: Codec<P1, OT, O>,
        P1R,
        NxtField: MapFieldGetter<P1, P1C, Struct, P1R, OT, O>,
        Struct,
    >(
        self,
        field: NxtField,
    ) -> MapCodecBuilder<MapCodec1<P1, P1C, P1R, NxtField, Struct, OT, O>, OT, O> {
        MapCodecBuilder {
            codec: MapCodec1 {
                codec1: field,
                into_struct: OnceCell::new(),
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
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
                $codec: Codec<$name, OT, O>,
                $field_return_type,
                $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type, OT, O>
            ),*
            , OT: Clone, O: CodecOps<OT>, Struct
        > MapCodecBuilder<
            $type<
                $(
                    $name,
                    $codec,
                    $field_return_type,
                    $field_type
                ),*
                , Struct, OT, O
            >, OT, O
        > {
            pub fn field<
                $next_name,
                $next_codec: Codec<$next_name, OT, O>,
                $next_field_return_type,
                NxtField: MapFieldGetter<
                    $next_name,
                    $next_codec,
                    Struct,
                    $next_field_return_type, OT, O>>
            (
                self,
                field: NxtField
            ) -> MapCodecBuilder<
                $next_type<
                    $($name, $codec, $field_return_type, $field_type),*,
                    $next_name, $next_codec, $next_field_return_type, NxtField, Struct, OT, O
                >, OT, O
            > {
                MapCodecBuilder {
                    codec: $next_type {
                        $($field: self.codec.$field),*,
                        $next_field_name: field,
                        into_struct: OnceCell::new(),
                        _phantom: PhantomData,
                    },
                    _phantom: PhantomData
                }
            }

            pub fn build(self, into_struct: fn($($field_return_type),*) -> Struct) -> impl Codec<Struct, OT, O> {
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
        #[doc(hidden)]
        impl<$($name, $codec: Codec<$name, OT, O>, $field_return_type, $field_type: MapFieldGetter<$name, $codec, Struct, $field_return_type, OT, O>),*, Struct, OT: Clone, O: CodecOps<OT>>
            MapCodecBuilder<$type<$($name, $codec, $field_return_type, $field_type),*, Struct, OT, O>, OT, O> {
            pub fn build(self, into_struct: fn($($field_return_type),*) -> Struct) -> impl Codec<Struct, OT, O> {
                self.codec.into_struct.set(into_struct).unwrap();
                self.codec
            }
        }
    };
}

impl_record_codec_builder! {
    type: MapCodec1,
    fields: { codec1: P1[P1C; P1F; P1R] },
    next: codec2: MapCodec2 as P2[P2C; P2F; P2R]
}

impl_record_codec_builder! {
    type: MapCodec2,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R] },
    next: codec3: MapCodec3 as P3[P3C; P3F; P3R]
}
impl_record_codec_builder! {
    type: MapCodec3,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R] },
    next: codec4: MapCodec4 as P4[P4C; P4F; P4R]
}

impl_record_codec_builder! {
    type: MapCodec4,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R] },
    next: codec5: MapCodec5 as P5[P5C; P5F; P5R]
}

impl_record_codec_builder! {
    type: MapCodec5,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R] },
    next: codec6: MapCodec6 as P6[P6C; P6F; P6R]
}

impl_record_codec_builder! {
    type: MapCodec6,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R] },
    next: codec7: MapCodec7 as P7[P7C; P7F; P7R]
}

impl_record_codec_builder! {
    type: MapCodec7,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R] },
    next: codec8: MapCodec8 as P8[P8C; P8F; P8R]
}

impl_record_codec_builder! {
    type: MapCodec8,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R] },
    next: codec9: MapCodec9 as P9[P9C; P9F; P9R]
}

impl_record_codec_builder! {
    type: MapCodec9,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R], codec9: P9[P9C; P9F; P9R] },
    next: codec10: MapCodec10 as P10[P10C; P10F; P10R]
}

impl_record_codec_builder_last! {
    type: MapCodec10,
    fields: { codec1: P1[P1C; P1F; P1R], codec2: P2[P2C; P2F; P2R], codec3: P3[P3C; P3F; P3R], codec4: P4[P4C; P4F; P4R], codec5: P5[P5C; P5F; P5R], codec6: P6[P6C; P6F; P6R], codec7: P7[P7C; P7F; P7R], codec8: P8[P8C; P8F; P8R], codec9: P9[P9C; P9F; P9R], codec10: P10[P10C; P10F; P10R] }
}
