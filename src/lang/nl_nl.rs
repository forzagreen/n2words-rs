//! Dutch (Netherlands) (`nl-NL`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `nl-NL`.
pub static LANGUAGE: Language = Language {
    code: "nl-NL",
    name: "Dutch (Netherlands)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
