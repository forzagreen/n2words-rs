//! German (Germany) (`de-DE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `de-DE`.
pub static LANGUAGE: Language = Language {
    code: "de-DE",
    name: "German (Germany)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
