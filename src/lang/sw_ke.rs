//! Swahili (Kenya) (`sw-KE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sw-KE`.
pub static LANGUAGE: Language = Language {
    code: "sw-KE",
    name: "Swahili (Kenya)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
