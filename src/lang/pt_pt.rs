//! European Portuguese (`pt-PT`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pt-PT`.
pub static LANGUAGE: Language = Language {
    code: "pt-PT",
    name: "European Portuguese",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
