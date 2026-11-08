//! Indonesian (Indonesia) (`id-ID`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `id-ID`.
pub static LANGUAGE: Language = Language {
    code: "id-ID",
    name: "Indonesian (Indonesia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
