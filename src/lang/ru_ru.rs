//! Russian (Russia) (`ru-RU`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ru-RU`.
pub static LANGUAGE: Language = Language {
    code: "ru-RU",
    name: "Russian (Russia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
