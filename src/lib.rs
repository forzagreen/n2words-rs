//! Numbers to words in 50 languages (72 regional variants).
//!
//! A Rust port of the [n2words](https://github.com/forzagreen/n2words)
//! JavaScript library. Each language is a module named after its BCP 47 code
//! in snake_case, exposing up to three **forms**:
//!
//! | Form     | Function                         | Example                             |
//! |----------|----------------------------------|-------------------------------------|
//! | Cardinal | [`en::to_cardinal`]              | "forty-two"                         |
//! | Ordinal  | [`en::to_ordinal`]               | "forty-second"                      |
//! | Currency | [`en_us::to_currency`]           | "forty-two dollars and fifty cents" |
//!
//! ```
//! use n2words::{en, en_us, es, fr, Currency};
//!
//! assert_eq!(en::to_cardinal(42)?, "forty-two");
//! assert_eq!(en::to_ordinal(42)?, "forty-second");
//! assert_eq!(es::to_cardinal(42)?, "cuarenta y dos");
//! assert_eq!(fr::to_cardinal("1234.5")?, "mille deux cent trente-quatre virgule cinq");
//!
//! // A region-qualified code carries a default currency ...
//! assert_eq!(en_us::to_currency(42.5)?, "forty-two dollars and fifty cents");
//! // ... while a bare language tag has no country to take one from, so it is explicit.
//! assert_eq!(en::to_currency(42.5, Currency::GBP)?, "forty-two pounds and fifty pence");
//! # Ok::<(), n2words::Error>(())
//! ```
//!
//! # Input
//!
//! Every function takes `impl Into<`[`Number`]`>`: any primitive integer, an
//! `f64`/`f32`, a numeric `&str`/`String` (arbitrary precision, scientific
//! notation accepted) or a [`num_bigint::BigInt`]/`BigUint`. Cardinal and
//! currency accept negatives and decimals; ordinal takes positive integers.
//!
//! # Options
//!
//! A form that has options comes in two flavours: `to_cardinal(value)` with
//! the language's defaults, and `to_cardinal_with(value, &options)` taking a
//! per-language options struct whose `Default` *is* those defaults:
//!
//! ```
//! use n2words::en_us::{self, CardinalOptions};
//!
//! assert_eq!(en_us::to_cardinal(1501)?, "one thousand five hundred one");
//! let informal = CardinalOptions { and: true, hundred_pairing: true, ..Default::default() };
//! assert_eq!(en_us::to_cardinal_with(1501, &informal)?, "fifteen hundred and one");
//! # Ok::<(), n2words::Error>(())
//! ```
//!
//! # Range
//!
//! Each form spells values up to the largest scale word it knows, declared as
//! a [`Max`] constant (`en_us::CARDINAL_MAX` is `10^66`), and returns
//! [`Error::OutOfRange`] past it rather than inventing vocabulary.
//!
//! # Picking a language at runtime
//!
//! [`language()`] looks an entry point up by code and gives access to the same
//! conversions through string-keyed options, which is what the `n2words`
//! command-line tool is built on:
//!
//! ```
//! use n2words::{language, FormKind};
//!
//! let de = language("de").expect("German is supported");
//! assert_eq!(de.convert(FormKind::Cardinal, 42, &[])?, "zweiundvierzig");
//! assert_eq!(de.convert(FormKind::Currency, "42.50", &[("currency", "EUR")])?, "zweiundvierzig Euro und fünfzig Cent");
//! # Ok::<(), n2words::Error>(())
//! ```
//!
//! # Languages
//!
//! Most languages are reachable through a bare primary subtag (`de`, `en`,
//! `fr`, ...), a documented alias of one specific regional variant; the
//! region- or script-qualified module always exists too (`en_gb`, `fr_be`),
//! and is the only entry point for the few languages whose variants differ in
//! script or core numbering (`zh_hans_cn`/`zh_hant_tw`, `pt_br`/`pt_pt`,
//! `sr_cyrl_rs`/`sr_latn_rs`, `am_et`/`am_latn_et`). See `LANGUAGES.md` in
//! the repository for the full table, per-language options and ceilings.

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/n2words/0.1.1")]

pub use num_bigint;

pub mod currency;
mod error;
mod lang;
pub mod language;
mod max;
mod number;
pub mod options;

pub use currency::Currency;
pub use error::{Error, Result};
pub use lang::*;
pub use language::{ConvertFn, Form, FormKind, Kind, Language, language, languages};
pub use max::Max;
pub use number::Number;
pub use options::Gender;

/// The crate version, as published.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
