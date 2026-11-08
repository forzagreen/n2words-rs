//! Latvian (Latvia) (`lv-LV`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `lv-LV`.
pub static LANGUAGE: Language = Language {
    code: "lv-LV",
    name: "Latvian (Latvia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
