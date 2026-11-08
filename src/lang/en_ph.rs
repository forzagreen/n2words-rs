//! English (Philippines) (`en-PH`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-PH`.
pub static LANGUAGE: Language = Language {
    code: "en-PH",
    name: "English (Philippines)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
