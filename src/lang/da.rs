//! Danish (`da`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `da`.
pub static LANGUAGE: Language = Language {
    code: "da",
    name: "Danish",
    kind: Kind::Alias { target: "da-DK" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
