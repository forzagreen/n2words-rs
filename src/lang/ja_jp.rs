//! Japanese (Japan) (`ja-JP`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ja-JP`.
pub static LANGUAGE: Language = Language {
    code: "ja-JP",
    name: "Japanese (Japan)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
