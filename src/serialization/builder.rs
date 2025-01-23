use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use super::{
    Codec,
    record::{RecordCodec1, RecordCodec2, RecordField, UnitCodec},
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

impl<P1, P1C: Codec<P1>, Struct> RecordCodecBuilder<RecordCodec1<P1, P1C, Struct>> {
    pub fn field<P2, P2C: Codec<P2>>(
        self,
        codec: P2C,
        field_name: impl Into<String>,
        getter: fn(&Struct) -> &P2,
    ) -> RecordCodecBuilder<RecordCodec2<P1, P1C, P2, P2C, Struct>> {
        RecordCodecBuilder {
            codec: RecordCodec2 {
                codec1: self.codec.codec1,
                codec2: RecordField {
                    field_name: field_name.into(),
                    getter,
                    codec,
                    _phantom: PhantomData,
                },
                into_struct: OnceCell::new(),
            },
        }
    }

    pub fn build(self, into_struct: fn(P1) -> Struct) -> impl Codec<Struct> {
        self.codec.into_struct.set(into_struct).unwrap();
        self.codec
    }
}

impl<P1, P1C: Codec<P1>, P2, P2C: Codec<P2>, Struct>
    RecordCodecBuilder<RecordCodec2<P1, P1C, P2, P2C, Struct>>
{
    pub fn build(self, into_struct: fn(P1, P2) -> Struct) -> impl Codec<Struct> {
        self.codec.into_struct.set(into_struct).unwrap();
        self.codec
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dynamic::Dynamic,
        serialization::{Codec, DefaultCodec, builder::RecordCodecBuilder},
    };

    #[derive(Debug, PartialEq)]
    struct Pos2d {
        x: f64,
        y: f64,
    }

    impl Pos2d {
        fn new(x: f64, y: f64) -> Pos2d {
            Pos2d { x, y }
        }
        fn x(&self) -> &f64 {
            &self.x
        }

        fn y(&self) -> &f64 {
            &self.y
        }

        fn codec() -> impl Codec<Pos2d> {
            RecordCodecBuilder::new()
                .field(f64::codec(), "x", Pos2d::x)
                .field(f64::codec(), "y", Pos2d::y)
                .build(Pos2d::new)
        }
    }

    #[test]
    pub fn simple_record() {
        let value = Pos2d { x: 10.0, y: 15.0 };
        let encoded = Pos2d::codec().encode(&Dynamic::ops(), &value).unwrap();
        let decoded = Pos2d::codec().decode(&Dynamic::ops(), &encoded).unwrap();

        assert_eq!(decoded, value);
    }
}
