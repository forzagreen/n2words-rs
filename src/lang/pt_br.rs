//! Brazilian Portuguese (`pt-BR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `pt-BR`.
pub static LANGUAGE: Language = Language {
    code: "pt-BR",
    name: "Brazilian Portuguese",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
