use std::fmt::Debug;

use datafix::{
    result::{CodecResult, DataError},
    serialization::{
        Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder, json::JsonOps,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum WebEvent {
    PageLoad,
    PageUnload,
    Click(ClickPos),
    Key(String),
}

impl WebEvent {
    pub fn leak_string(str: &str) -> &'static String {
        Box::leak(Box::new(str.to_string()))
    }

    pub fn event(&self) -> &String {
        match self {
            WebEvent::PageLoad => Self::leak_string("page_load"),
            WebEvent::PageUnload => Self::leak_string("page_unload"),
            WebEvent::Click(_) => Self::leak_string("click"),
            WebEvent::Key(_) => Self::leak_string("key"),
        }
    }
}

impl WebEvent {
    pub fn page_load_codec<S: CodecOps>() -> impl Codec<Self, S> {
        MapCodecBuilder::new()
            .field(
                Codecs::constant(String::codec(), "page_load").field_of("event", WebEvent::event),
            )
            .build(|_| WebEvent::PageLoad)
    }

    pub fn page_unload_codec<S: CodecOps>() -> impl Codec<Self, S> {
        MapCodecBuilder::new()
            .field(
                Codecs::constant(String::codec(), "page_unload").field_of("event", WebEvent::event),
            )
            .build(|_| WebEvent::PageUnload)
    }

    pub fn click_event<S: CodecOps>() -> impl Codec<Self, S> {
        MapCodecBuilder::new()
            .field(Codecs::constant(String::codec(), "click").field_of("event", WebEvent::event))
            .field(ClickPos::codec().fallible_field_of("pos", |x| match x {
                WebEvent::Click(pos) => Ok(pos),
                _ => Err(DataError::new_custom("WebEvent::Event")),
            }))
            .build(|_, pos| WebEvent::Click(pos))
    }

    pub fn key_event<S: CodecOps>() -> impl Codec<Self, S> {
        MapCodecBuilder::new()
            .field(Codecs::constant(String::codec(), "key").field_of("event", WebEvent::event))
            .field(String::codec().fallible_field_of("value", |x| match x {
                WebEvent::Key(key) => Ok(key),
                _ => Err(DataError::new_custom("WebEvent::Key")),
            }))
            .build(|_, key| WebEvent::Key(key))
    }

    pub fn debug_codec<S: CodecOps>() -> impl Codec<Self, S> {
        Self::key_event()
            .try_else(Self::click_event())
            .try_else(Self::page_load_codec())
            .try_else(Self::page_unload_codec())
    }
}

impl<S: CodecOps> DefaultCodec<S> for WebEvent {
    fn codec() -> impl Codec<Self, S> {
        WebEvent::debug_codec::<S>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickPos {
    x: i64,
    y: i64,
}

impl<S: CodecOps> DefaultCodec<S> for ClickPos {
    fn codec() -> impl datafix::serialization::Codec<Self, S> {
        MapCodecBuilder::new()
            .field(i64::codec().field_of("x", |v: &ClickPos| &v.x))
            .field(i64::codec().field_of("y", |v: &ClickPos| &v.y))
            .build(|x, y| ClickPos { x, y })
    }
}

fn main() -> CodecResult<()> {
    let event = WebEvent::Click(ClickPos { x: 12, y: 39 });
    let codec = WebEvent::codec();
    println!("codec: {}", codec.debug());

    let encoded = codec.encode_start(&JsonOps, &event)?;
    println!("encoded: {}", encoded);
    let decoded = codec.decode_start(&JsonOps, &encoded)?;

    assert_eq!(event, decoded);
    Ok(())
}
