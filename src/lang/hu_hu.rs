//! Hungarian (Hungary) (`hu-HU`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hu-HU`.
pub static LANGUAGE: Language = Language {
    code: "hu-HU",
    name: "Hungarian (Hungary)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
