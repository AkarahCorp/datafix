//! `datafix` serializes and deserializes data of different versions.
//!
//! This crate does two major things:
//!
//! * **Serialization and deserialization** Datafixer provides a built-in serialization method [`serialization::Codec`].
//! This is a different approach from Serde as Serde uses a recursive visitor approach through macros, while Datafixer
//! uses data & traits to generate the code needed.
//!
//! * **Data transformations** `todo!()`
//!
//! For example, let's say you're developing a text editor and want to provide a configuration.
//!
//! ```rs
//! struct Config {
//!     font_size: i32,
//!     font: String
//! }
//! ```
//! You can create a declarative way of serializing & deserializing this data using `Codec`s.
//! ```rs
//! impl DefaultCodec for Config {
//!     fn codec() -> impl Codec<Config> {
//!         StructCodecBuilder::new()
//!             .field(i32::codec(), "font_size", Config::font_size)
//!             .field(String::codec(), "font", Config::font)
//!             .apply(Config::new)
//!     }
//! }
//! ```
//! However, you may eventually want to upgrade this configuration to have more data.
//! ```rs
//! struct Config {
//!     font_size: i32,
//!     font: String,
//!     exit_key: String
//! }
//! ```
//! You ideally want your user's configuration to be automatically updated with the new data and a sensible default.
//! This part is currently `todo!()`, sorry. :(
#![no_std]

extern crate alloc;

/// The module for all things related to fixing up data.
pub mod fixers;
/// The module that contains `DataError`, the error value returned by methods related to this crate when things go wrong.
pub mod result;
/// The module responsible for transforming data between different types and representations.
pub mod serialization;
