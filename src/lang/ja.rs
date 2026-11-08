//! Japanese (`ja`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ja`.
pub static LANGUAGE: Language = Language {
    code: "ja",
    name: "Japanese",
    kind: Kind::Alias { target: "ja-JP" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
