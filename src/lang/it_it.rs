//! Italian (Italy) (`it-IT`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `it-IT`.
pub static LANGUAGE: Language = Language {
    code: "it-IT",
    name: "Italian (Italy)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
