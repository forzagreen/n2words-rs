//! Hausa (`ha`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ha`.
pub static LANGUAGE: Language = Language {
    code: "ha",
    name: "Hausa",
    kind: Kind::Alias { target: "ha-NG" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
