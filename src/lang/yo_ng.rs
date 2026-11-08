//! Yoruba (Nigeria) (`yo-NG`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `yo-NG`.
pub static LANGUAGE: Language = Language {
    code: "yo-NG",
    name: "Yoruba (Nigeria)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
