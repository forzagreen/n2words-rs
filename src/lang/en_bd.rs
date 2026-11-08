//! English (Bangladesh) (`en-BD`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-BD`.
pub static LANGUAGE: Language = Language {
    code: "en-BD",
    name: "English (Bangladesh)",
    kind: Kind::Profile { base: "en-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
