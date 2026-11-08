//! Marathi (India) (`mr-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `mr-IN`.
pub static LANGUAGE: Language = Language {
    code: "mr-IN",
    name: "Marathi (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
