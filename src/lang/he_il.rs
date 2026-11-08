//! Hebrew (Israel) (`he-IL`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `he-IL`.
pub static LANGUAGE: Language = Language {
    code: "he-IL",
    name: "Hebrew (Israel)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
