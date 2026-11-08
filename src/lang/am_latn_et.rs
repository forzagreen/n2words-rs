//! Amharic (Latin, Ethiopia) (`am-Latn-ET`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `am-Latn-ET`.
pub static LANGUAGE: Language = Language {
    code: "am-Latn-ET",
    name: "Amharic (Latin, Ethiopia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
