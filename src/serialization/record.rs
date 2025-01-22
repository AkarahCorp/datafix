use core::{cell::OnceCell, marker::PhantomData};

use alloc::string::String;

use crate::{
    dynamic::{Dynamic, object::DynamicObject},
    result::{DataError, DataResult},
};

use super::Codec;

pub struct RecordField<T, C: Codec<T>, S> {
    pub(crate) field_name: String,
    pub(crate) getter: fn(&S) -> &T,
    pub(crate) codec: C,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

pub struct UnitCodec {}

impl Codec<()> for UnitCodec {
    fn into_dyn(&self, _: &()) -> DataResult<Dynamic> {
        Ok(Dynamic::new(DynamicObject::new()))
    }

    fn from_dyn(&self, value: Dynamic) -> DataResult<()> {
        let Dynamic::Object(_) = value else {
            return Err(DataError::new("expected object of {}"));
        };
        Ok(())
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
    fn into_dyn(&self, value: &Struct) -> DataResult<Dynamic> {
        let mut object = DynamicObject::new();
        object.insert(
            self.codec1.field_name.clone(),
            self.codec1.codec.into_dyn(&(self.codec1.getter)(value))?,
        );
        Ok(Dynamic::new(object))
    }

    fn from_dyn(&self, mut value: Dynamic) -> DataResult<Struct> {
        let Some(object) = value.as_object_mut() else {
            return Err(DataError::new("expected Object"));
        };

        let Some(p1) = object.remove(&self.codec1.field_name) else {
            return Err(DataError::new("expected Object with p1"));
        };

        Ok((self.into_struct.get().unwrap())(
            self.codec1.codec.from_dyn(p1)?,
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
    fn into_dyn(&self, value: &Struct) -> DataResult<Dynamic> {
        let mut object = DynamicObject::new();
        object.insert(
            self.codec1.field_name.clone(),
            self.codec1.codec.into_dyn(&(self.codec1.getter)(&value))?,
        );
        object.insert(
            self.codec2.field_name.clone(),
            self.codec2.codec.into_dyn(&(self.codec2.getter)(&value))?,
        );
        Ok(Dynamic::new(object))
    }

    fn from_dyn(&self, mut value: Dynamic) -> DataResult<Struct> {
        let Some(object) = value.as_object_mut() else {
            return Err(DataError::new("expected Object"));
        };

        let Some(p1) = object.remove(&self.codec1.field_name) else {
            return Err(DataError::new("expected Object with p1"));
        };

        let Some(p2) = object.remove(&self.codec2.field_name) else {
            return Err(DataError::new("expected Object with p2"));
        };

        Ok((self.into_struct.get().unwrap())(
            self.codec1.codec.from_dyn(p1)?,
            self.codec2.codec.from_dyn(p2)?,
        ))
    }
}
