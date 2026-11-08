//! English (India) (`en-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-IN`.
pub static LANGUAGE: Language = Language {
    code: "en-IN",
    name: "English (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
