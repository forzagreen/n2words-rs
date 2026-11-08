//! German (`de`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `de`.
pub static LANGUAGE: Language = Language {
    code: "de",
    name: "German",
    kind: Kind::Alias { target: "de-DE" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
