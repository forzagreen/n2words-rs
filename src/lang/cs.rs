//! Czech (`cs`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `cs`.
pub static LANGUAGE: Language = Language {
    code: "cs",
    name: "Czech",
    kind: Kind::Alias { target: "cs-CZ" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
