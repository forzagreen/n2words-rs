//! Biblical Hebrew (Israel) (`hbo-IL`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hbo-IL`.
pub static LANGUAGE: Language = Language {
    code: "hbo-IL",
    name: "Biblical Hebrew (Israel)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
