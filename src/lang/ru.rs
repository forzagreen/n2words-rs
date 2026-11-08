//! Russian (`ru`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ru`.
pub static LANGUAGE: Language = Language {
    code: "ru",
    name: "Russian",
    kind: Kind::Alias { target: "ru-RU" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
