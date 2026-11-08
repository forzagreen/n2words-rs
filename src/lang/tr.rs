//! Turkish (`tr`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `tr`.
pub static LANGUAGE: Language = Language {
    code: "tr",
    name: "Turkish",
    kind: Kind::Alias { target: "tr-TR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
