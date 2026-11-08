//! Polish (`pl`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pl`.
pub static LANGUAGE: Language = Language {
    code: "pl",
    name: "Polish",
    kind: Kind::Alias { target: "pl-PL" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
