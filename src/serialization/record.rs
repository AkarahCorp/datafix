use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use crate::result::{DataError, DataResult};

use super::{Codec, ops::CodecOps};

pub struct RecordField<T, C: Codec<T>, S> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&S) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
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

pub struct RecordCodec1<P1, P1C, Struct>
where
    P1C: Codec<P1>,
{
    pub(crate) codec1: RecordField<P1, P1C, Struct>,
    pub(crate) into_struct: OnceCell<fn(P1) -> Struct>,
}

impl<P1, P1C, Struct> Codec<Struct> for RecordCodec1<P1, P1C, Struct>
where
    P1C: Codec<P1>,
{
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
        Ok(ops.create_object(&[(
            &self.codec1.field_name,
            self.codec1.codec.encode(ops, (self.codec1.getter)(value))?,
        )]))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Struct> {
        let obj = ops.get_object(value)?;
        let Some(p1) = obj.get(&self.codec1.field_name) else {
            return Err(DataError::new(""));
        };
        Ok((self.into_struct.get().unwrap())(
            self.codec1.codec.decode(ops, p1)?,
        ))
    }
}

pub struct RecordCodec2<P1, P1C, P2, P2C, Struct>
where
    P1C: Codec<P1>,
    P2C: Codec<P2>,
{
    pub(crate) codec1: RecordField<P1, P1C, Struct>,
    pub(crate) codec2: RecordField<P2, P2C, Struct>,
    pub(crate) into_struct: OnceCell<fn(P1, P2) -> Struct>,
}

impl<P1, P1C, P2, P2C, Struct> Codec<Struct> for RecordCodec2<P1, P1C, P2, P2C, Struct>
where
    P1C: Codec<P1>,
    P2C: Codec<P2>,
{
    fn encode<U, O: CodecOps<U>>(&self, ops: &O, value: &Struct) -> DataResult<U> {
        Ok(ops.create_object(&[
            (
                &self.codec1.field_name,
                self.codec1.codec.encode(ops, (self.codec1.getter)(value))?,
            ),
            (
                &self.codec2.field_name,
                self.codec2.codec.encode(ops, (self.codec2.getter)(value))?,
            ),
        ]))
    }

    fn decode<U, O: CodecOps<U>>(&self, ops: &O, value: &U) -> DataResult<Struct> {
        let obj = ops.get_object(value)?;
        let Some(p1) = obj.get(&self.codec1.field_name) else {
            return Err(DataError::new(""));
        };
        let Some(p2) = obj.get(&self.codec2.field_name) else {
            return Err(DataError::new(""));
        };
        Ok((self.into_struct.get().unwrap())(
            self.codec1.codec.decode(ops, p1)?,
            self.codec2.codec.decode(ops, p2)?,
        ))
    }
}
