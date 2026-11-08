//! Ukrainian (`uk`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `uk`.
pub static LANGUAGE: Language = Language {
    code: "uk",
    name: "Ukrainian",
    kind: Kind::Alias { target: "uk-UA" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
