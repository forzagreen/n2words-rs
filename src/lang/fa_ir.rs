//! Persian (Iran) (`fa-IR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fa-IR`.
pub static LANGUAGE: Language = Language {
    code: "fa-IR",
    name: "Persian (Iran)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
