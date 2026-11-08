//! Kannada (India) (`kn-IN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `kn-IN`.
pub static LANGUAGE: Language = Language {
    code: "kn-IN",
    name: "Kannada (India)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
