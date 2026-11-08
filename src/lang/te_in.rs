//! Telugu (India) (`te-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `te-IN`.
pub static LANGUAGE: Language = Language {
    code: "te-IN",
    name: "Telugu (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
