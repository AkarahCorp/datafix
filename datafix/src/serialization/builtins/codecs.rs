use core::{fmt::Debug, marker::PhantomData, ops::RangeBounds};

use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use either::Either;

use crate::{
    result::{DataError, DataResult},
    serialization::{Codec, CodecAdapters, CodecOps, DefaultCodec, ListView, MapView},
};

pub(crate) struct StringCodec;

impl<O: CodecOps> Codec<String, O> for StringCodec {
    fn encode(&self, ops: &O, value: &String) -> DataResult<O::T> {
        Ok(ops.create_string(value))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<String> {
        ops.get_string(value)
    }
}

impl<O: CodecOps> DefaultCodec<O> for String {
    fn codec() -> impl Codec<Self, O> {
        StringCodec
    }
}

pub(crate) struct BoolCodec;

impl<O: CodecOps> Codec<bool, O> for BoolCodec {
    fn encode(&self, ops: &O, value: &bool) -> DataResult<O::T> {
        Ok(ops.create_boolean(value))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<bool> {
        ops.get_boolean(value)
    }
}

impl<O: CodecOps> DefaultCodec<O> for bool {
    fn codec() -> impl Codec<Self, O> {
        BoolCodec
    }
}

pub(crate) struct ListCodec<T, C: Codec<T, O>, O: CodecOps> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, O)>,
}

impl<T, C: Codec<T, O>, O: CodecOps> Codec<Vec<T>, O> for ListCodec<T, C, O> {
    fn encode(&self, ops: &O, value: &Vec<T>) -> DataResult<O::T> {
        let mut list = Vec::new();
        for element in value {
            list.push(self.inner.encode(ops, element)?);
        }
        Ok(ops.create_list(list))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<Vec<T>> {
        let list = ops.get_list(value)?;
        let mut vec = Vec::new();
        for item in list.into_iter() {
            vec.push(self.inner.decode(ops, &item)?);
        }
        Ok(vec)
    }
}

pub(crate) struct XMapCodec<OLT, NT, C, F1, F2, O: CodecOps>
where
    C: Codec<OLT, O>,
    F1: Fn(&OLT) -> NT,
    F2: Fn(&NT) -> OLT,
{
    pub(crate) inner: C,
    pub(crate) f1: F1,
    pub(crate) f2: F2,
    pub(crate) _phantom: PhantomData<fn() -> (OLT, NT, O)>,
}

impl<OLT, NT, C, F1, F2, O: CodecOps> Codec<NT, O> for XMapCodec<OLT, NT, C, F1, F2, O>
where
    C: Codec<OLT, O>,
    F1: Fn(&OLT) -> NT,
    F2: Fn(&NT) -> OLT,
{
    fn encode(&self, ops: &O, value: &NT) -> DataResult<O::T> {
        self.inner.encode(ops, &(self.f2)(value))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<NT> {
        Ok((self.f1)(&self.inner.decode(ops, value)?))
    }
}

pub(crate) struct PairCodec<L, R, Lc: Codec<L, O>, Rc: Codec<R, O>, O: CodecOps> {
    pub(crate) left: Lc,
    pub(crate) right: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (L, R, O)>,
}
impl<L, R, Lc: Codec<L, O>, Rc: Codec<R, O>, O: CodecOps> Codec<(L, R), O>
    for PairCodec<L, R, Lc, Rc, O>
{
    fn encode(&self, ops: &O, value: &(L, R)) -> DataResult<O::T> {
        Ok(ops.create_map([
            ("left".to_string(), self.left.encode(ops, &value.0)?),
            ("right".to_string(), self.right.encode(ops, &value.1)?),
        ]))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<(L, R)> {
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
    C: Codec<T, O>,
    R: RangeBounds<T>,
    O: CodecOps,
> {
    pub(crate) codec: C,
    pub(crate) range: R,
    pub(crate) _phantom: PhantomData<fn() -> (T, O)>,
}

impl<T: PartialOrd + Debug, C: Codec<T, O>, R: RangeBounds<T>, O: CodecOps> Codec<T, O>
    for BoundedCodec<T, C, R, O>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
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

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
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

pub struct DynamicCodec<T, O: CodecOps> {
    pub(crate) codec: Box<dyn Codec<T, O>>,
}

impl<T, O: CodecOps> Codec<T, O> for DynamicCodec<T, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct ArcCodec<T, O: CodecOps> {
    pub(crate) codec: Arc<dyn Codec<T, O>>,
}

impl<T, O: CodecOps> Clone for ArcCodec<T, O> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
        }
    }
}

impl<T, O: CodecOps> Codec<T, O> for ArcCodec<T, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        self.codec.as_ref().encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        self.codec.as_ref().decode(ops, value)
    }
}

pub struct FnCodec<T, O: CodecOps> {
    pub(crate) encode: Box<dyn Fn(&O, &T) -> DataResult<O::T>>,
    pub(crate) decode: Box<dyn Fn(&O, &O::T) -> DataResult<T>>,
}

impl<T, O: CodecOps> Codec<T, O> for FnCodec<T, O> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        (self.encode)(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        (self.decode)(ops, value)
    }
}

pub struct BoxCodec<T, O: CodecOps, C: Codec<T, O>> {
    pub(crate) inner: C,
    pub(crate) _phantom: PhantomData<fn() -> (T, O)>,
}

impl<T, O: CodecOps, C: Codec<T, O>> Codec<Box<T>, O> for BoxCodec<T, O, C> {
    fn encode(&self, ops: &O, value: &Box<T>) -> DataResult<O::T> {
        self.inner.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<Box<T>> {
        self.inner.decode(ops, value).map(|x| Box::new(x))
    }
}

pub struct TryElseCodec<T, O: CodecOps, Lc: Codec<T, O>, Rc: Codec<T, O>> {
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, O)>,
}

impl<T, O: CodecOps, Lc: Codec<T, O>, Rc: Codec<T, O>> Codec<T, O> for TryElseCodec<T, O, Lc, Rc> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        self.lc
            .encode(ops, value)
            .or_else(|_| self.rc.encode(ops, value))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        self.lc
            .decode(ops, value)
            .or_else(|_| self.rc.decode(ops, value))
    }
}

pub struct EitherCodec<T, O: CodecOps, T2, Lc: Codec<T, O>, Rc: Codec<T2, O>> {
    pub(crate) lc: Lc,
    pub(crate) rc: Rc,
    pub(crate) _phantom: PhantomData<fn() -> (T, O, T2)>,
}

impl<T, O: CodecOps, T2, Lc: Codec<T, O>, Rc: Codec<T2, O>> Codec<Either<T, T2>, O>
    for EitherCodec<T, O, T2, Lc, Rc>
{
    fn encode(&self, ops: &O, value: &Either<T, T2>) -> DataResult<O::T> {
        match value {
            Either::Left(value) => self.lc.encode(ops, value),
            Either::Right(value) => self.rc.encode(ops, value),
        }
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<Either<T, T2>> {
        match self.lc.decode(ops, value) {
            Ok(v) => Ok(Either::Left(v)),
            Err(_) => match self.rc.decode(ops, value) {
                Ok(v) => Ok(Either::Right(v)),
                Err(e) => Err(e),
            },
        }
    }
}

pub struct OrElseCodec<T, O: CodecOps, C: Codec<T, O>, F: Fn() -> T> {
    pub(crate) codec: C,
    pub(crate) default: F,
    pub(crate) _phantom: PhantomData<fn() -> (T, O)>,
}

impl<T, O: CodecOps, C: Codec<T, O>, F: Fn() -> T> Codec<T, O> for OrElseCodec<T, O, C, F> {
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        self.codec.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        Ok(self
            .codec
            .decode(ops, value)
            .unwrap_or_else(|_| (self.default)()))
    }
}

pub struct DispatchCodec<
    T,
    O: CodecOps,
    E: Fn(&T) -> DataResult<DynamicCodec<T, O>>,
    F: Fn(&O, &O::T) -> DataResult<DynamicCodec<T, O>>,
> {
    pub(crate) from_type_to_codec: E,
    pub(crate) from_ops_to_codec: F,
    pub(crate) _phantom: PhantomData<(T, O)>,
}

impl<
    T,
    O: CodecOps,
    E: Fn(&T) -> DataResult<DynamicCodec<T, O>>,
    F: Fn(&O, &O::T) -> DataResult<DynamicCodec<T, O>>,
> Codec<T, O> for DispatchCodec<T, O, E, F>
{
    fn encode(&self, ops: &O, value: &T) -> DataResult<O::T> {
        (self.from_type_to_codec)(value)?.encode(ops, value)
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<T> {
        (self.from_ops_to_codec)(ops, value)?.decode(ops, value)
    }
}

macro_rules! make_numeric_codec {
    (
        $({$t:ty, $struct_name:ident, $get_name:ident, $make_name:ident})*
        $(;)?
    ) => {
        $(pub struct $struct_name<O: CodecOps> {
            _phantom: PhantomData<fn() -> O>,
        }

        impl<O: CodecOps> Codec<$t, O> for $struct_name<O> {
            fn encode(&self, ops: &O, value: &$t) -> DataResult<O::T> {
                Ok(ops.$make_name(value))
            }

            fn decode(&self, ops: &O, value: &O::T) -> DataResult<$t> {
                ops.$get_name(value)
            }
        }

        impl<O: CodecOps> DefaultCodec<O> for $t {
            fn codec() -> impl Codec<Self, O> {
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

macro_rules! make_unsigned_codec {
    (
        $(
            {$from:ty; $to:ty}
        )*
    ) => {
        $(impl<O: CodecOps> DefaultCodec<O> for $to {
            fn codec() -> impl Codec<Self, O> {
                <$from>::codec().xmap(|x| *x as $to, |x| *x as $from)
            }
        })*
    };
}

make_unsigned_codec! {
    {i8; u8}
    {i16; u16}
    {i32; u32}
    {i64; u64}
}

pub struct UntypedMapCodec<T, O: CodecOps, C: Codec<T, O>> {
    codec: C,
    _phantom: PhantomData<(T, O, C)>,
}

impl<T, O: CodecOps, C: Codec<T, O>> Codec<BTreeMap<String, T>, O> for UntypedMapCodec<T, O, C> {
    fn encode(&self, ops: &O, value: &BTreeMap<String, T>) -> DataResult<O::T> {
        let entries = value
            .iter()
            .map(|x| (x.0.clone(), self.codec.encode(ops, x.1).unwrap()))
            .collect::<Vec<_>>();
        Ok(ops.create_map(entries))
    }

    fn decode(&self, ops: &O, value: &O::T) -> DataResult<BTreeMap<String, T>> {
        let mut map = BTreeMap::new();
        let Ok(view) = ops.get_map(value) else {
            return Ok(map);
        };
        for key in view.keys() {
            let value = view.get(&key)?;
            let decode = self.codec.decode(ops, value)?;
            map.insert(key, decode);
        }
        Ok(map)
    }
}

impl<T: DefaultCodec<O>, O: CodecOps> DefaultCodec<O> for BTreeMap<String, T> {
    fn codec() -> impl Codec<Self, O> {
        UntypedMapCodec {
            codec: T::codec(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        boxed::Box,
        collections::btree_map::BTreeMap,
        string::{String, ToString},
        vec,
    };
    use json::JsonValue;

    use crate::{
        result::DataError,
        serialization::{
            Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder,
            builtins::codecs::{ArcCodec, DynamicCodec},
            json::JsonOps,
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
    pub fn default_codec() {
        #[derive(Clone, Debug, PartialEq)]
        struct Wrapper {
            value: f64,
        }

        let codec = MapCodecBuilder::new()
            .field(f64::codec().default_field_of("value", |w: &Wrapper| &w.value, || 12.1))
            .build(|value| Wrapper { value });

        let value = Wrapper { value: 0.0 };
        let encoded = codec.encode(&JsonOps, &value).unwrap();
        let decoded = codec.decode(&JsonOps, &encoded).unwrap();
        assert_eq!(value, decoded);

        let empty_obj = JsonValue::new_object();
        let decoded = codec.decode(&JsonOps, &empty_obj).unwrap();
        assert_eq!(Wrapper { value: 12.1 }, decoded);
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

        impl<O: CodecOps> DefaultCodec<O> for LinkedList
        where
            ArcCodec<LinkedList, JsonOps>: Codec<LinkedList, O>,
        {
            fn codec() -> impl Codec<Self, O> {
                Codecs::recursive(|codec: DynamicCodec<LinkedList, JsonOps>| {
                    MapCodecBuilder::new()
                        .field(i32::codec().field_of("value", |l: &LinkedList| &l.value))
                        .field(
                            codec
                                .boxed()
                                .optional_field_of("next", |l: &LinkedList| &l.next),
                        )
                        .build(|value, next| LinkedList { value, next })
                })
            }
        }

        let value = LinkedList::new(1).seq(LinkedList::new(2).seq(LinkedList::new(3)));

        let codec = LinkedList::codec();

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
            pub fn number_codec<O: CodecOps + 'static>() -> DynamicCodec<Self, O> {
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

            pub fn string_codec<O: CodecOps + 'static>() -> DynamicCodec<Self, O> {
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

            pub fn codec<O: CodecOps + 'static>() -> impl Codec<Self, O> {
                Codecs::dispatch(
                    |value| match value {
                        UnknownType::Number(_) => Ok(UnknownType::number_codec()),
                        UnknownType::String(_) => Ok(UnknownType::string_codec()),
                    },
                    |ops: &O, value: &O::T| {
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

    #[test]
    fn untyped_map_codec() {
        let mut map = BTreeMap::new();
        map.insert("x".to_string(), 10.0);
        map.insert("y".to_string(), 20.0);
        map.insert("z".to_string(), 30.0);

        let encoded = BTreeMap::codec().encode(&JsonOps, &map).unwrap();
        let decoded = BTreeMap::codec().decode(&JsonOps, &encoded).unwrap();

        assert_eq!(map, decoded);
    }
}
