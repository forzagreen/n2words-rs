//! Marathi (`mr`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `mr`.
pub static LANGUAGE: Language = Language {
    code: "mr",
    name: "Marathi",
    kind: Kind::Alias { target: "mr-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
