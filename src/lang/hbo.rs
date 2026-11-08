//! Biblical Hebrew (`hbo`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hbo`.
pub static LANGUAGE: Language = Language {
    code: "hbo",
    name: "Biblical Hebrew",
    kind: Kind::Alias { target: "hbo-IL" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
