//! Ukrainian (Ukraine) (`uk-UA`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `uk-UA`.
pub static LANGUAGE: Language = Language {
    code: "uk-UA",
    name: "Ukrainian (Ukraine)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
