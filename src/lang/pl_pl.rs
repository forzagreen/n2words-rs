//! Polish (Poland) (`pl-PL`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pl-PL`.
pub static LANGUAGE: Language = Language {
    code: "pl-PL",
    name: "Polish (Poland)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
