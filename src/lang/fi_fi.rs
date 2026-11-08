//! Finnish (Finland) (`fi-FI`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fi-FI`.
pub static LANGUAGE: Language = Language {
    code: "fi-FI",
    name: "Finnish (Finland)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
