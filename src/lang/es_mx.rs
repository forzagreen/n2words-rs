//! Mexican Spanish (`es-MX`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `es-MX`.
pub static LANGUAGE: Language = Language {
    code: "es-MX",
    name: "Mexican Spanish",
    kind: Kind::Profile { base: "es-ES" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
