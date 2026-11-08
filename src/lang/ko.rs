//! Korean (`ko`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ko`.
pub static LANGUAGE: Language = Language {
    code: "ko",
    name: "Korean",
    kind: Kind::Alias { target: "ko-KR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
