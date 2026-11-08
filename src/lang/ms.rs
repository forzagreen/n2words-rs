//! Malay (`ms`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ms`.
pub static LANGUAGE: Language = Language {
    code: "ms",
    name: "Malay",
    kind: Kind::Alias { target: "ms-MY" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
