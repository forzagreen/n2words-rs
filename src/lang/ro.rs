//! Romanian (`ro`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ro`.
pub static LANGUAGE: Language = Language {
    code: "ro",
    name: "Romanian",
    kind: Kind::Alias { target: "ro-RO" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
