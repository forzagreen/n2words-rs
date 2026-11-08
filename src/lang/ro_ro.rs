//! Romanian (Romania) (`ro-RO`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ro-RO`.
pub static LANGUAGE: Language = Language {
    code: "ro-RO",
    name: "Romanian (Romania)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
