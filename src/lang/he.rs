//! Hebrew (`he`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `he`.
pub static LANGUAGE: Language = Language {
    code: "he",
    name: "Hebrew",
    kind: Kind::Alias { target: "he-IL" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
