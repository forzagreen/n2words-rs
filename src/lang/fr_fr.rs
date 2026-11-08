//! French (France) (`fr-FR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fr-FR`.
pub static LANGUAGE: Language = Language {
    code: "fr-FR",
    name: "French (France)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
