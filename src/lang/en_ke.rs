//! English (Kenya) (`en-KE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-KE`.
pub static LANGUAGE: Language = Language {
    code: "en-KE",
    name: "English (Kenya)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
