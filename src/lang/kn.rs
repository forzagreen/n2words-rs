//! Kannada (`kn`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `kn`.
pub static LANGUAGE: Language = Language {
    code: "kn",
    name: "Kannada",
    kind: Kind::Alias { target: "kn-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
