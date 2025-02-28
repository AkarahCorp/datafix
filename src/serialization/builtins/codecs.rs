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

pub(crate) struct StringCodec;

impl<U: Clone, O: CodecOps<U>> Codec<String, U, O> for StringCodec {
    fn encode(&self, ops: &O, value: &String) -> DataResult<U> {
        Ok(ops.create_string(value))
    }

    fn decode(&self, ops: &O, value: &U) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl<U: Clone, O: CodecOps<U>> DefaultCodec<U, O> for String {
    fn codec() -> impl Codec<Self, U, O> {
        StringCodec
    }
}

pub(crate) struct BoolCodec;

impl<U: Clone, O: CodecOps<U>> Codec<bool, U, O> for BoolCodec {
    fn encode(&self, ops: &O, value: &bool) -> DataResult<U> {
        Ok(ops.create_boolean(value))
    }

    fn decode(&self, ops: &O, value: &U) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}

impl<U: Clone, O: CodecOps<U>> DefaultCodec<U, O> for bool {
    fn codec() -> impl Codec<Self, U, O> {
        BoolCodec
    }
}

pub(crate) struct ListCodec<T, C: Codec<T, U, O>, U: Clone, O: CodecOps<U>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, U, O)>,
}

impl<T, C: Codec<T, U, O>, U: Clone, O: CodecOps<U>> Codec<Vec<T>, U, O> for ListCodec<T, C, U, O> {
    fn encode(&self, ops: &O, value: &Vec<T>) -> DataResult<U> {
        let mut list = Vec::new();
        for element in value {
            list.push(self.inner.encode(ops, element)?);
        }
        Ok(ops.create_list(list))
    }

    fn decode(&self, ops: &O, value: &U) -> DataResult<Vec<T>> {
        let list = ops.get_list(value)?;
        let mut vec = Vec::new();
        for item in list.into_iter() {
            vec.push(self.inner.decode(ops, &item)?);
        }
        Ok(vec)
    }
}

pub(crate) struct XMapCodec<OLT, NT, C, F1, F2, U: Clone, O: CodecOps<U>>
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

impl<OLT, NT, C, F1, F2, OT: Clone, O: CodecOps<OT>> Codec<NT, OT, O>
    for XMapCodec<OLT, NT, C, F1, F2, OT, O>
where
    C: Codec<OLT, OT, O>,
    F1: Fn(&OLT) -> NT,
    F2: Fn(&NT) -> OLT,
{
    fn encode(&self, ops: &O, value: &NT) -> DataResult<OT> {
        self.inner.encode(ops, &(self.f2)(value))
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<NT> {
        Ok((self.f1)(&self.inner.decode(ops, value)?))
    }
}

pub(crate) struct PairCodec<
    L,
    R,
    Lc: Codec<L, OT, O>,
    Rc: Codec<R, OT, O>,
    OT: Clone,
    O: CodecOps<OT>,
> {
    pub(crate) left: Lc,
    pub(crate) right: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (L, R, OT, O)>,
}
impl<L, R, Lc: Codec<L, OT, O>, Rc: Codec<R, OT, O>, OT: Clone, O: CodecOps<OT>>
    Codec<(L, R), OT, O> for PairCodec<L, R, Lc, Rc, OT, O>
{
    fn encode(&self, ops: &O, value: &(L, R)) -> DataResult<OT> {
        Ok(ops.create_map([
            ("left".to_string(), self.left.encode(ops, &value.0)?),
            ("right".to_string(), self.right.encode(ops, &value.1)?),
        ]))
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<(L, R)> {
        let obj = ops.get_map(value)?;
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
    OT: Clone,
    O: CodecOps<OT>,
> {
    pub(crate) codec: C,
    pub(crate) range: R,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T: PartialOrd + Debug, C: Codec<T, OT, O>, R: RangeBounds<T>, OT: Clone, O: CodecOps<OT>>
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

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
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

pub struct DynamicCodec<T, OT: Clone, O: CodecOps<OT>> {
    pub(crate) codec: Box<dyn Codec<T, OT, O>>,
}

impl<T, OT: Clone, O: CodecOps<OT>> Codec<T, OT, O> for DynamicCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct ArcCodec<T, OT: Clone, O: CodecOps<OT>> {
    pub(crate) codec: Arc<dyn Codec<T, OT, O>>,
}

impl<T, OT: Clone, O: CodecOps<OT>> Clone for ArcCodec<T, OT, O> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
        }
    }
}

impl<T, OT: Clone, O: CodecOps<OT>> Codec<T, OT, O> for ArcCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct FnCodec<T, OT: Clone, O: CodecOps<OT>> {
    pub(crate) encode: Box<dyn Fn(&O, &T) -> DataResult<OT>>,
    pub(crate) decode: Box<dyn Fn(&O, &OT) -> DataResult<T>>,
}

impl<T, OT: Clone, O: CodecOps<OT>> Codec<T, OT, O> for FnCodec<T, OT, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        (self.encode)(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        (self.decode)(ops, value)
    }
}

pub struct BoxCodec<T, OT: Clone, O: CodecOps<OT>, C: Codec<T, OT, O>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T, OT: Clone, O: CodecOps<OT>, C: Codec<T, OT, O>> Codec<Box<T>, OT, O>
    for BoxCodec<T, OT, O, C>
{
    fn encode(&self, ops: &O, value: &Box<T>) -> DataResult<OT> {
        self.inner.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<Box<T>> {
        self.inner.decode(ops, value).map(|x| Box::new(x))
    }
}

pub struct TryElseCodec<T, OT: Clone, O: CodecOps<OT>, Lc: Codec<T, OT, O>, Rc: Codec<T, OT, O>> {
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T, OT: Clone, O: CodecOps<OT>, Lc: Codec<T, OT, O>, Rc: Codec<T, OT, O>> Codec<T, OT, O>
    for TryElseCodec<T, OT, O, Lc, Rc>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.lc
            .encode(ops, value)
            .or_else(|_| self.rc.encode(ops, value))
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        self.lc
            .decode(ops, value)
            .or_else(|_| self.rc.decode(ops, value))
    }
}

pub struct EitherCodec<T, OT: Clone, O: CodecOps<OT>, T2, Lc: Codec<T, OT, O>, Rc: Codec<T2, OT, O>>
{
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O, T2)>,
}

impl<T, OT: Clone, O: CodecOps<OT>, T2, Lc: Codec<T, OT, O>, Rc: Codec<T2, OT, O>>
    Codec<Either<T, T2>, OT, O> for EitherCodec<T, OT, O, T2, Lc, Rc>
{
    fn encode(&self, ops: &O, value: &Either<T, T2>) -> DataResult<OT> {
        match value {
            Either::Left(value) => self.lc.encode(ops, value),
            Either::Right(value) => self.rc.encode(ops, value),
        }
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<Either<T, T2>> {
        match self.lc.decode(ops, value) {
            Ok(v) => Ok(Either::Left(v)),
            Err(_) => match self.rc.decode(ops, value) {
                Ok(v) => Ok(Either::Right(v)),
                Err(e) => Err(e),
            },
        }
    }
}

pub struct OrElseCodec<T, OT: Clone, O: CodecOps<OT>, C: Codec<T, OT, O>, F: Fn() -> T> {
    pub(crate) codec: C,
    pub(crate) default: F,
    pub(crate) _phantom: PhantomData<fn() -> (T, OT, O)>,
}

impl<T, OT: Clone, O: CodecOps<OT>, C: Codec<T, OT, O>, F: Fn() -> T> Codec<T, OT, O>
    for OrElseCodec<T, OT, O, C, F>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        self.codec.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        Ok(self
            .codec
            .decode(ops, value)
            .unwrap_or_else(|_| (self.default)()))
    }
}

pub struct DispatchCodec<
    T,
    OT: Clone,
    O: CodecOps<OT>,
    E: Fn(&T) -> DataResult<DynamicCodec<T, OT, O>>,
    F: Fn(&O, &OT) -> DataResult<DynamicCodec<T, OT, O>>,
> {
    pub(crate) from_type_to_codec: E,
    pub(crate) from_ops_to_codec: F,
    pub(crate) _phantom: PhantomData<(T, OT, O)>,
}

impl<
    T,
    OT: Clone,
    O: CodecOps<OT>,
    E: Fn(&T) -> DataResult<DynamicCodec<T, OT, O>>,
    F: Fn(&O, &OT) -> DataResult<DynamicCodec<T, OT, O>>,
> Codec<T, OT, O> for DispatchCodec<T, OT, O, E, F>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<OT> {
        (self.from_type_to_codec)(value)?.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &OT) -> DataResult<T> {
        (self.from_ops_to_codec)(ops, value)?.decode(ops, value)
    }
}

macro_rules! make_numeric_codec {
    (
        $({$t:ty, $struct_name:ident, $get_name:ident, $make_name:ident})*
        $(;)?
    ) => {
        $(pub struct $struct_name<OT: Clone, O: CodecOps<OT>> {
            _phantom: PhantomData<fn() -> (OT, O)>,
        }

        impl<OT: Clone, O: CodecOps<OT>> Codec<$t, OT, O> for $struct_name<OT, O> {
            fn encode(&self, ops: &O, value: &$t) -> DataResult<OT> {
                Ok(ops.$make_name(value))
            }

            fn decode(&self, ops: &O, value: &OT) -> DataResult<$t> {
                ops.$get_name(value)
            }
        }

        impl<OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O> for $t {
            fn codec() -> impl Codec<Self, OT, O> {
                $struct_name {
                    _phantom: PhantomData,
                }
            }
        })*
    };
}

make_numeric_codec! {
    {f32, F32Codec, get_float, create_float}
    {f64, F64Codec, get_double, create_double}

    {i8, I8Codec, get_byte, create_byte}
    {i16, I16Codec, get_short, create_short}
    {i32, I32Codec, get_int, create_int}
    {i64, I64Codec, get_long, create_long}
}

#[cfg(test)]
mod tests {
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        vec,
    };

    use crate::{
        result::DataError,
        serialization::{
            Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder,
            builtins::codecs::DynamicCodec, json::JsonOps,
        },
    };

    #[test]
    fn f64_codec() {
        let value = 10.0;
        let encoded = f64::codec().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn string_codec() {
        let value = "Hello!".into();
        let encoded = String::codec().encode(&JsonOps, &value).unwrap();
        let decoded = String::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn bool_codec() {
        let value = true;
        let encoded = bool::codec().encode(&JsonOps, &value).unwrap();
        let decoded = bool::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn numeric_codec() {
        let value = 10;
        let encoded = i32::codec().encode(&JsonOps, &value).unwrap();
        let decoded = i32::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);

        let value = 10;
        let encoded = i64::codec().encode(&JsonOps, &value).unwrap();
        let decoded = i64::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn list_codec() {
        let value = vec![10, 20, 30];
        let encoded = i32::codec().list_of().encode(&JsonOps, &value).unwrap();
        let decoded = i32::codec().list_of().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn xmap_codec() {
        let value = 15;
        let codec = i32::codec().xmap(|x| x * 5, |x| x / 5);
        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn pair_codec() {
        let value = (15, "Hello".to_string());
        let codec = i32::codec().pair(String::codec());
        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn bounded_codec() {
        let value = 15;
        let codec = i32::codec().bounded(1..30);
        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);

        assert!(codec.encode(&JsonOps, &75).is_err());
        assert!(codec.encode(&JsonOps, &1).is_ok());
        assert!(codec.encode(&JsonOps, &30).is_err());
    }

    #[test]
    fn dynamic_codec() {
        let value = 10.0;
        let encoded = f64::codec().dynamic().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec().dynamic().decode(&JsonOps, &encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn arc_codec() {
        let value = 10.0;
        let encoded = f64::codec().arc().encode(&JsonOps, &value).unwrap();
        let decoded = f64::codec().dynamic().decode(&JsonOps, &encoded).unwrap();
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
        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();
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

        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    pub fn dispatch_codec() {
        #[derive(PartialEq, Debug)]
        enum UnknownType {
            Number(f64),
            String(String),
        }

        impl UnknownType {
            pub fn number_codec<OT: 'static + Clone, O: CodecOps<OT> + 'static>()
            -> DynamicCodec<Self, OT, O> {
                f64::codec()
                    .xmap(
                        |x| UnknownType::Number(*x),
                        |x| {
                            let UnknownType::Number(x) = x else {
                                panic!();
                            };
                            *x
                        },
                    )
                    .dynamic()
            }

            pub fn string_codec<OT: 'static + Clone, O: CodecOps<OT> + 'static>()
            -> DynamicCodec<Self, OT, O> {
                String::codec()
                    .xmap(
                        |x| UnknownType::String(x.clone()),
                        |x| {
                            let UnknownType::String(x) = x else {
                                unreachable!();
                            };
                            x.clone()
                        },
                    )
                    .dynamic()
            }

            pub fn codec<OT: 'static + Clone, O: CodecOps<OT> + 'static>() -> impl Codec<Self, OT, O>
            {
                Codecs::dispatch(
                    |value: &UnknownType| match value {
                        UnknownType::Number(_) => Ok(UnknownType::number_codec()),
                        UnknownType::String(_) => Ok(UnknownType::string_codec()),
                    },
                    |ops: &O, value: &OT| {
                        if ops.get_string(value).is_ok() {
                            Ok(UnknownType::string_codec())
                        } else if ops.get_double(value).is_ok() {
                            Ok(UnknownType::number_codec())
                        } else {
                            Err(DataError::unexpected_type("string | number"))
                        }
                    },
                )
            }
        }

        let value = UnknownType::Number(10.0);
        let encoded = UnknownType::codec().encode(&JsonOps, &value).unwrap();
        let decoded = UnknownType::codec().decode(&JsonOps, &encoded).unwrap();
        assert_eq!(value, decoded);

        let value = UnknownType::String("foobar".to_string());
        let encoded = UnknownType::codec().encode(&JsonOps, &value).unwrap();
        let decoded = UnknownType::codec().decode(&JsonOps, &encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
