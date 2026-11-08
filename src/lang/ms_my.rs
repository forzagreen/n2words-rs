//! Malay (Malaysia) (`ms-MY`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ms-MY`.
pub static LANGUAGE: Language = Language {
    code: "ms-MY",
    name: "Malay (Malaysia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
