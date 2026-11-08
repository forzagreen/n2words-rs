//! Greek (Greece) (`el-GR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `el-GR`.
pub static LANGUAGE: Language = Language {
    code: "el-GR",
    name: "Greek (Greece)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
