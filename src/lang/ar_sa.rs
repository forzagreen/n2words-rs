//! Arabic (Saudi Arabia) (`ar-SA`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ar-SA`.
pub static LANGUAGE: Language = Language {
    code: "ar-SA",
    name: "Arabic (Saudi Arabia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
