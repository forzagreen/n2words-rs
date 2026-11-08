//! Telugu (`te`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `te`.
pub static LANGUAGE: Language = Language {
    code: "te",
    name: "Telugu",
    kind: Kind::Alias { target: "te-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
