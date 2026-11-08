//! Punjabi (`pa`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pa`.
pub static LANGUAGE: Language = Language {
    code: "pa",
    name: "Punjabi",
    kind: Kind::Alias { target: "pa-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
