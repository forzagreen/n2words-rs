//! Danish (Denmark) (`da-DK`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `da-DK`.
pub static LANGUAGE: Language = Language {
    code: "da-DK",
    name: "Danish (Denmark)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
