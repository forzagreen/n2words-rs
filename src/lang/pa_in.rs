//! Punjabi (India) (`pa-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pa-IN`.
pub static LANGUAGE: Language = Language {
    code: "pa-IN",
    name: "Punjabi (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
