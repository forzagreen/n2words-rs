//! Dutch (`nl`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `nl`.
pub static LANGUAGE: Language = Language {
    code: "nl",
    name: "Dutch",
    kind: Kind::Alias { target: "nl-NL" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
