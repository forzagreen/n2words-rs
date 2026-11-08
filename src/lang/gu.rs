//! Gujarati (`gu`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `gu`.
pub static LANGUAGE: Language = Language {
    code: "gu",
    name: "Gujarati",
    kind: Kind::Alias { target: "gu-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
