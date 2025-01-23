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
    ) -> RecordCodecBuilder<RecordCodec1<P1, P1C, RecordField<P1, P1C, Struct>, Struct>> {
        RecordCodecBuilder {
            codec: RecordCodec1 {
                codec1: RecordField {
                    field_name: field_name.into(),
                    getter,
                    codec,
                    _phantom: PhantomData,
                },
                into_struct: OnceCell::new(),
                _phantom: PhantomData,
            },
        }
    }
}

macro_rules! impl_record_codec_builder {
    (
        type: $type:ident,
        fields: { $($field:ident: $name:ident[$codec:ident, $field_type:ident]),* },
        next: $next_field_name:ident: $next_type:ident as $next_name:ident[$next_codec:ident]
    ) => {
        impl<$($name, $codec: Codec<$name>, $field_type: RecordFieldGetter<$name, $codec, Struct>),*, Struct> RecordCodecBuilder<$type<$($name, $codec, $field_type),*, Struct>> {
            pub fn field<$next_name, $next_codec: Codec<$next_name>>(
                self,
                codec: $next_codec,
                field_name: impl Into<String>,
                getter: fn(&Struct) -> &$next_name,
            ) -> RecordCodecBuilder<$next_type<$($name, $codec, $field_type),*, $next_name, $next_codec, RecordField<$next_name, $next_codec, Struct>, Struct>> {
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
    fields: { codec1: P1[P1C, P1F] },
    next: codec2: RecordCodec2 as P2[P2C]
}

impl_record_codec_builder_last! {
    type: RecordCodec2,
    fields: { codec1: P1[P1C; P1F], codec2: P2[P2C; P2F] }
}
