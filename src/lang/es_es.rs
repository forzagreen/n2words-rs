//! European Spanish (`es-ES`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `es-ES`.
pub static LANGUAGE: Language = Language {
    code: "es-ES",
    name: "European Spanish",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
