//! Korean (South Korea) (`ko-KR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ko-KR`.
pub static LANGUAGE: Language = Language {
    code: "ko-KR",
    name: "Korean (South Korea)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
