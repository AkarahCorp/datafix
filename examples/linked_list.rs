use datafix::{
    result::CodecResult,
    serialization::{
        Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder, json::JsonOps,
    },
};
use json::JsonValue;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Nested {
    value: i32,
    next: Option<Box<Nested>>,
}

impl Nested {
    pub fn new(value: i32) -> Self {
        Nested { value, next: None }
    }

    pub fn with(mut self, value: i32) -> Self {
        match self.next {
            Some(next) => self.next = Some(Box::new(next.with(value))),
            None => self.next = Some(Box::new(Nested::new(value))),
        }
        self
    }
}

impl<O: CodecOps + 'static> DefaultCodec<O> for Nested {
    fn codec() -> impl datafix::serialization::Codec<Self, O> {
        Codecs::recursive(|codec| {
            MapCodecBuilder::new()
                .field(i32::codec().field_of("value", |x: &Nested| &x.value))
                .field(
                    codec
                        .boxed()
                        .optional_field_of("next", |x: &Nested| &x.next),
                )
                .build(|value, next| Nested { value, next })
        })
    }
}

fn main() -> CodecResult<()> {
    let list = Nested::new(10).with(20).with(30);
    let mut encoded = Nested::codec().encode_start(&JsonOps, &list)?;
    println!("{}", encoded.pretty(4));
    if let JsonValue::Object(object) = &mut encoded {
        if let JsonValue::Object(object) = object.get_mut("next").unwrap() {
            if let JsonValue::Object(object) = object.get_mut("next").unwrap() {
                object.insert("value", "hi".into());
            }
        }
    }
    let decoded = Nested::codec().decode_start(&JsonOps, &encoded)?;
    assert_eq!(list, decoded);
    Ok(())
}
