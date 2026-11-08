//! Czech (Czechia) (`cs-CZ`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `cs-CZ`.
pub static LANGUAGE: Language = Language {
    code: "cs-CZ",
    name: "Czech (Czechia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
