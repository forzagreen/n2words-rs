//! English (Malaysia) (`en-MY`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-MY`.
pub static LANGUAGE: Language = Language {
    code: "en-MY",
    name: "English (Malaysia)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
