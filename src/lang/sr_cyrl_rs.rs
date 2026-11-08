//! Serbian (Cyrillic, Serbia) (`sr-Cyrl-RS`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sr-Cyrl-RS`.
pub static LANGUAGE: Language = Language {
    code: "sr-Cyrl-RS",
    name: "Serbian (Cyrillic, Serbia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
