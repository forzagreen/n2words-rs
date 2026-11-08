//! Arabic (`ar`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ar`.
pub static LANGUAGE: Language = Language {
    code: "ar",
    name: "Arabic",
    kind: Kind::Alias { target: "ar-SA" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
