//! Latvian (`lv`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `lv`.
pub static LANGUAGE: Language = Language {
    code: "lv",
    name: "Latvian",
    kind: Kind::Alias { target: "lv-LV" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
