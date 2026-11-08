//! English (South Africa) (`en-ZA`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-ZA`.
pub static LANGUAGE: Language = Language {
    code: "en-ZA",
    name: "English (South Africa)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
