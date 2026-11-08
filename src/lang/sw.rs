//! Swahili (`sw`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sw`.
pub static LANGUAGE: Language = Language {
    code: "sw",
    name: "Swahili",
    kind: Kind::Alias { target: "sw-KE" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
