//! Bangla (Bangladesh) (`bn-BD`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `bn-BD`.
pub static LANGUAGE: Language = Language {
    code: "bn-BD",
    name: "Bangla (Bangladesh)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
