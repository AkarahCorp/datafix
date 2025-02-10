use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use either::Either;

use crate::{
    result::{DataError, DataResult},
    serialization::{Codec, CodecOps, DefaultCodec, ListView, MapView},
};

pub(crate) struct F64Codec;

impl<OT, O: CodecOps<OT>> Codec<f64, OT, O> for F64Codec {
    fn encode(&self, ops: &O, value: &f64) -> DataResult<OT> {
        Ok(ops.create_number(value))
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<f64> {
        ops.get_number(value)
    }
}

impl<U, O: CodecOps<U>> DefaultCodec<U, O> for f64 {
    fn codec() -> impl Codec<Self, U, O> {
        F64Codec
    }
}

pub(crate) struct StringCodec;

impl<U, O: CodecOps<U>> Codec<String, U, O> for StringCodec {
    fn encode(&self, ops: &O, value: &String) -> DataResult<U> {
        Ok(ops.create_string(value))
    }

    fn decode(&self, ops: &O, value: &mut U) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl<U, O: CodecOps<U>> DefaultCodec<U, O> for String {
    fn codec() -> impl Codec<Self, U, O> {
        StringCodec
    }
}

pub(crate) struct BoolCodec;

impl<U, O: CodecOps<U>> Codec<bool, U, O> for BoolCodec {
    fn encode(&self, ops: &O, value: &bool) -> DataResult<U> {
        Ok(ops.create_boolean(value))
    }

    fn decode(&self, ops: &O, value: &mut U) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}

impl<U, O: CodecOps<U>> DefaultCodec<U, O> for bool {
    fn codec() -> impl Codec<Self, U, O> {
        BoolCodec
    }
}

pub(crate) trait F64Convertable
where
    Self: Sized + Copy,
{
    fn into_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! impl_f64_convertable {
    ($($t:ty),*) => {
        $(
            impl F64Convertable for $t {
                fn into_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(value: f64) -> Self {
                    value as $t
                }
            }

            impl<U, O: CodecOps<U>> DefaultCodec<U, O> for $t {
                fn codec() -> impl Codec<Self, U, O> {
                    NumberCodec {
                        _phantom: PhantomData,
                    }
                }
            }
        )*
    };
}

impl_f64_convertable! { i8, i16, i32, i64, u8, u16, u32, u64, f32, usize, isize }

pub(crate) struct NumberCodec<N: F64Convertable, U, O: CodecOps<U>> {
    _phantom: PhantomData<fn() -> (N, U, O)>,
}

impl<U, O: CodecOps<U>, N: F64Convertable> Codec<N, U, O> for NumberCodec<N, U, O> {
    fn encode(&self, ops: &O, value: &N) -> DataResult<U> {
        Ok(ops.create_number(&value.into_f64()))
    }

    fn decode(&self, ops: &O, value: &mut U) -> DataResult<N> {
        Ok(N::from_f64(ops.get_number(value)?))
    }
}

pub(crate) struct ListCodec<T, C: Codec<T, U, O>, U, O: CodecOps<U>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, U, O)>,
}

impl<T, C: Codec<T, U, O>, U, O: CodecOps<U>> Codec<Vec<T>, U, O> for ListCodec<T, C, U, O> {
    fn encode(&self, ops: &O, value: &Vec<T>) -> DataResult<U> {
        let mut list = Vec::new();
        for element in value {
            list.push(self.inner.encode(ops, element)?);
        }
        Ok(ops.create_list(list))
    }

    fn decode(&self, ops: &O, value: &mut U) -> DataResult<Vec<T>> {
        let list = ops.get_list(value)?;
        let mut vec = Vec::new();
        for mut item in list.into_iter() {
            vec.push(self.inner.decode(ops, &mut item)?);
        }
        Ok(vec)
    }
}

pub(crate) struct XMapCodec<OLT, NT, C, F1, F2, U, O: CodecOps<U>>
where
    C: Codec<OLT, U, O>,
    F1: Fn(&OLT) -> NT,
    F2: Fn(&NT) -> OLT,
{
    pub(crate) inner: C,
    pub(crate) f1: F1,
    pub(crate) f2: F2,
    pub(crate) _phantom: PhantomData<fn() -> (OLT, NT, U, O)>,
}

impl<OLT, NT, C, F1, F2, OT, O: CodecOps<OT>> Codec<NT, OT, O>
    for XMapCodec<OLT, NT, C, F1, F2, OT, O>
where
    C: Codec<OLT, OT, O>,
    F1: Fn(&OLT) -> NT,
    F2: Fn(&NT) -> OLT,
{
    fn encode(&self, ops: &O, value: &NT) -> DataResult<OT> {
        self.inner.encode(ops, &(self.f2)(value))
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<NT> {
        Ok((self.f1)(&self.inner.decode(ops, value)?))
    }
}

pub(crate) struct PairCodec<L, R, Lc: Codec<L, OT, O>, Rc: Codec<R, OT, O>, OT, O: CodecOps<OT>> {
    pub(crate) left: Lc,
    pub(crate) right: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (L, R, OT, O)>,
}
impl<L, R, Lc: Codec<L, OT, O>, Rc: Codec<R, OT, O>, OT, O: CodecOps<OT>> Codec<(L, R), OT, O>
    for PairCodec<L, R, Lc, Rc, OT, O>
{
    fn encode(&self, ops: &O, value: &(L, R)) -> DataResult<OT> {
        Ok(ops.create_map([
            ("left".to_string(), self.left.encode(ops, &value.0)?),
            ("right".to_string(), self.right.encode(ops, &value.1)?),
        ]))
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<(L, R)> {
        let mut obj = ops.get_map(value)?;
        let left = obj.get("left")?;
        let p1 = self.left.decode(ops, left)?;
        let right = obj.get("right")?;
        let p2 = self.right.decode(ops, right)?;
        Ok((p1, p2))
    }
}

pub(crate) struct BoundedCodec<
    T: PartialOrd + Debug,
    C: Codec<T, OT, O>,
    R: RangeBounds<T>,
    OT,
    O: CodecOps<OT>,
> {
    pub(crate) codec: C,
    pub(crate) range: R,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T: PartialOrd + Debug, C: Codec<T, OT, O>, R: RangeBounds<T>, OT, O: CodecOps<OT>>
    Codec<T, OT, O> for BoundedCodec<T, C, R, OT, O>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        if !self.range.contains(value) {
            Err(DataError::new_custom(&alloc::format!(
                "value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        } else {
            self.codec.encode(ops, value)
        }
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        let decoded = self.codec.decode(ops, value)?;
        if self.range.contains(&decoded) {
            Ok(decoded)
        } else {
            Err(DataError::new_custom(&alloc::format!(
                "value must be in bounds of {:?} to {:?}",
                self.range.start_bound(),
                self.range.end_bound()
            )))
        }
    }
}

pub struct DynamicCodec<T, OT, O: CodecOps<OT>> {
    pub(crate) codec: Box<dyn Codec<T, OT, O>>,
}

impl<T, OT, O: CodecOps<OT>> Codec<T, OT, O> for DynamicCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct ArcCodec<T, OT, O: CodecOps<OT>> {
    pub(crate) codec: Arc<dyn Codec<T, OT, O>>,
}

impl<T, OT, O: CodecOps<OT>> Clone for ArcCodec<T, OT, O> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
        }
    }
}

impl<T, OT, O: CodecOps<OT>> Codec<T, OT, O> for ArcCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct FnCodec<T, OT, O: CodecOps<OT>> {
    pub(crate) encode: Box<dyn Fn(&O, &T) -> DataResult<OT>>,
    pub(crate) decode: Box<dyn Fn(&O, &mut OT) -> DataResult<T>>,
}

impl<T, OT, O: CodecOps<OT>> Codec<T, OT, O> for FnCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        (self.encode)(ops, value)
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        (self.decode)(ops, value)
    }
}

pub struct BoxCodec<T, OT, O: CodecOps<OT>, C: Codec<T, OT, O>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T, OT, O: CodecOps<OT>, C: Codec<T, OT, O>> Codec<Box<T>, OT, O> for BoxCodec<T, OT, O, C> {
    fn encode(&self, ops: &O, value: &Box<T>) -> DataResult<OT> {
        self.inner.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<Box<T>> {
        self.inner.decode(ops, value).map(|x| Box::new(x))
    }
}

pub struct TryElseCodec<T, OT, O: CodecOps<OT>, Lc: Codec<T, OT, O>, Rc: Codec<T, OT, O>> {
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>
}

impl<T, OT, O: CodecOps<OT>, Lc: Codec<T, OT, O>, Rc: Codec<T, OT, O>> Codec<T, OT, O> for TryElseCodec<T, OT, O, Lc, Rc> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.lc.encode(ops, value)
            .or_else(|_| self.rc.encode(ops, value))
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        self.lc.decode(ops, value)
            .or_else(|_| self.rc.decode(ops, value))
    }
}

pub struct EitherCodec<
    T, OT, O: CodecOps<OT>, 
    T2,
    Lc: Codec<T, OT, O>, 
    Rc: Codec<T2, OT, O>> {
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O, T2)>
}

impl<T, OT, O: CodecOps<OT>, 
    T2,
    Lc: Codec<T, OT, O>, Rc: Codec<T2, OT, O>> Codec<Either<T, T2>, OT, O> for EitherCodec<T, OT, O, T2, Lc, Rc> {
    fn encode(&self, ops: &O, value: &Either<T, T2>) -> DataResult<OT> {
        match value {
            Either::Left(value) => self.lc.encode(ops, value),
            Either::Right(value) => self.rc.encode(ops, value),
        }
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<Either<T, T2>> {
        match self.lc.decode(ops, value) {
            Ok(v) => Ok(Either::Left(v)),
            Err(_) => match self.rc.decode(ops, value) {
                Ok(v) => Ok(Either::Right(v)),
                Err(e) => Err(e)
            }
        }
    }
}

pub struct OrElseCodec<T, OT, O: CodecOps<OT>, C: Codec<T, OT, O>, F: Fn() -> T> {
    pub(crate) codec: C,
    pub(crate) default: F,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>
}

impl<T, OT, O: CodecOps<OT>, C: Codec<T, OT, O>, F: Fn() -> T> Codec<T, OT, O> for OrElseCodec<T, OT, O, C, F> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &mut OT) -> DataResult<T> {
        Ok(self.codec.decode(ops, value).unwrap_or_else(|_| (self.default)()))
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        vec,
    };

    use crate::serialization::{
        Codec, CodecAdapters, Codecs, DefaultCodec, MapCodecBuilder, json::JsonOps,
    };

    #[test]
    fn f64_codec() {
        let value = 10.0;
        let mut encoded = f64::codec().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec().decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn string_codec() {
        let value = "Hello!".into();
        let mut encoded = String::codec().encode(&JsonOps, &value).unwrap();
        let decoded = String::codec().decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn bool_codec() {
        let value = true;
        let mut encoded = bool::codec().encode(&JsonOps, &value).unwrap();
        let decoded = bool::codec().decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn numeric_codec() {
        let value = 10;
        let mut encoded = i32::codec().encode(&JsonOps, &value).unwrap();
        let decoded = i32::codec().decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);

        let value = 10;
        let mut encoded = i64::codec().encode(&JsonOps, &value).unwrap();
        let decoded = i64::codec().decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn list_codec() {
        let value = vec![10, 20, 30];
        let mut encoded = i32::codec().list_of().encode(&JsonOps, &value).unwrap();
        let decoded = i32::codec()
            .list_of()
            .decode(&JsonOps, &mut encoded)
            .unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn xmap_codec() {
        let value = 15;
        let codec = i32::codec().xmap(|x| x * 5, |x| x / 5);
        let mut encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn pair_codec() {
        let value = (15, "Hello".to_string());
        let codec = i32::codec().pair(String::codec());
        let mut encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn bounded_codec() {
        let value = 15;
        let codec = i32::codec().bounded(1..30);
        let mut encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);

        assert!(codec.encode(&JsonOps, &75).is_err());
        assert!(codec.encode(&JsonOps, &1).is_ok());
        assert!(codec.encode(&JsonOps, &30).is_err());
    }

    #[test]
    fn dynamic_codec() {
        let value = 10.0;
        let mut encoded = f64::codec().dynamic().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec()
            .dynamic()
            .decode(&JsonOps, &mut encoded)
            .unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn arc_codec() {
        let value = 10.0;
        let mut encoded = f64::codec().arc().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec()
            .dynamic()
            .decode(&JsonOps, &mut encoded)
            .unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    pub fn optional_codec() {
        #[derive(Clone, Debug, PartialEq)]
        struct Wrapper {
            value: Option<f64>,
        }

        let codec = MapCodecBuilder::new()
            .field(f64::codec().optional_field_of("value", |w: &Wrapper| &w.value))
            .build(|value| Wrapper { value });

        let value = Wrapper { value: None };
        let mut encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &mut encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    pub fn recursive_codec() {
        #[derive(Clone, PartialEq, Debug)]
        struct LinkedList {
            value: i32,
            next: Option<Box<LinkedList>>,
        }

        impl LinkedList {
            pub fn new(value: i32) -> LinkedList {
                LinkedList { value, next: None }
            }
            pub fn seq(self, next: LinkedList) -> Self {
                LinkedList {
                    value: self.value,
                    next: Some(Box::new(next)),
                }
            }
        }

        let value = LinkedList::new(1).seq(LinkedList::new(2).seq(LinkedList::new(3)));

        let codec = Codecs::recursive(|codec| {
            MapCodecBuilder::new()
                .field(i32::codec().field_of("value", |l: &LinkedList| &l.value))
                .field(
                    codec
                        .boxed()
                        .optional_field_of("next", |l: &LinkedList| &l.next),
                )
                .build(|value, next| LinkedList { value, next })
        });

        let mut encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &mut encoded).unwrap();

        assert_eq!(value, decoded);
    }
}
