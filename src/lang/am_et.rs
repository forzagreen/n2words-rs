//! Amharic (Ethiopia) (`am-ET`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `am-ET`.
pub static LANGUAGE: Language = Language {
    code: "am-ET",
    name: "Amharic (Ethiopia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
