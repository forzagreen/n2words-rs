//! Spanish (`es`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `es`.
pub static LANGUAGE: Language = Language {
    code: "es",
    name: "Spanish",
    kind: Kind::Alias { target: "es-ES" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
