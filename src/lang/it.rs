//! Italian (`it`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `it`.
pub static LANGUAGE: Language = Language {
    code: "it",
    name: "Italian",
    kind: Kind::Alias { target: "it-IT" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
