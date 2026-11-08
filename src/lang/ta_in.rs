//! Tamil (India) (`ta-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ta-IN`.
pub static LANGUAGE: Language = Language {
    code: "ta-IN",
    name: "Tamil (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
