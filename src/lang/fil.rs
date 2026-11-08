//! Filipino (`fil`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fil`.
pub static LANGUAGE: Language = Language {
    code: "fil",
    name: "Filipino",
    kind: Kind::Alias { target: "fil-PH" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
