//! Tamil (`ta`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ta`.
pub static LANGUAGE: Language = Language {
    code: "ta",
    name: "Tamil",
    kind: Kind::Alias { target: "ta-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
