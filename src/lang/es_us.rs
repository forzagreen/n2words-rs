//! Spanish (United States) (`es-US`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `es-US`.
pub static LANGUAGE: Language = Language {
    code: "es-US",
    name: "Spanish (United States)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
