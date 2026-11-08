//! Yoruba (`yo`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `yo`.
pub static LANGUAGE: Language = Language {
    code: "yo",
    name: "Yoruba",
    kind: Kind::Alias { target: "yo-NG" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
