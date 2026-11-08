//! Swedish (Sweden) (`sv-SE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sv-SE`.
pub static LANGUAGE: Language = Language {
    code: "sv-SE",
    name: "Swedish (Sweden)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
