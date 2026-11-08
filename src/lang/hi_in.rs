//! Hindi (India) (`hi-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hi-IN`.
pub static LANGUAGE: Language = Language {
    code: "hi-IN",
    name: "Hindi (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
