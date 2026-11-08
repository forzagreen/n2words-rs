//! Serbian (Latin, Serbia) (`sr-Latn-RS`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sr-Latn-RS`.
pub static LANGUAGE: Language = Language {
    code: "sr-Latn-RS",
    name: "Serbian (Latin, Serbia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
