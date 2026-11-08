//! Persian (`fa`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fa`.
pub static LANGUAGE: Language = Language {
    code: "fa",
    name: "Persian",
    kind: Kind::Alias { target: "fa-IR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
