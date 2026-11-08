//! Greek (`el`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `el`.
pub static LANGUAGE: Language = Language {
    code: "el",
    name: "Greek",
    kind: Kind::Alias { target: "el-GR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
