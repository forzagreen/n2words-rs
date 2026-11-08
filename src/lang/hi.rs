//! Hindi (`hi`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hi`.
pub static LANGUAGE: Language = Language {
    code: "hi",
    name: "Hindi",
    kind: Kind::Alias { target: "hi-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
