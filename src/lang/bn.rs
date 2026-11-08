//! Bangla (`bn`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `bn`.
pub static LANGUAGE: Language = Language {
    code: "bn",
    name: "Bangla",
    kind: Kind::Alias { target: "bn-BD" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
