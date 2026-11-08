//! Chinese (Traditional, Taiwan) (`zh-Hant-TW`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `zh-Hant-TW`.
pub static LANGUAGE: Language = Language {
    code: "zh-Hant-TW",
    name: "Chinese (Traditional, Taiwan)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
