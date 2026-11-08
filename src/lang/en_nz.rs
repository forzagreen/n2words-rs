//! English (New Zealand) (`en-NZ`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-NZ`.
pub static LANGUAGE: Language = Language {
    code: "en-NZ",
    name: "English (New Zealand)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
