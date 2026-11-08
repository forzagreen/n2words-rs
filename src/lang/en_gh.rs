//! English (Ghana) (`en-GH`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-GH`.
pub static LANGUAGE: Language = Language {
    code: "en-GH",
    name: "English (Ghana)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
