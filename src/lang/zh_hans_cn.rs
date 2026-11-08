//! Chinese (Simplified, China) (`zh-Hans-CN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `zh-Hans-CN`.
pub static LANGUAGE: Language = Language {
    code: "zh-Hans-CN",
    name: "Chinese (Simplified, China)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
