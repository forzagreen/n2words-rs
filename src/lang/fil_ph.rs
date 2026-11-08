//! Filipino (Philippines) (`fil-PH`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fil-PH`.
pub static LANGUAGE: Language = Language {
    code: "fil-PH",
    name: "Filipino (Philippines)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
