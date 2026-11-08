//! French (`fr`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fr`.
pub static LANGUAGE: Language = Language {
    code: "fr",
    name: "French",
    kind: Kind::Alias { target: "fr-FR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
