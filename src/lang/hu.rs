//! Hungarian (`hu`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hu`.
pub static LANGUAGE: Language = Language {
    code: "hu",
    name: "Hungarian",
    kind: Kind::Alias { target: "hu-HU" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
