//! Indonesian (`id`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `id`.
pub static LANGUAGE: Language = Language {
    code: "id",
    name: "Indonesian",
    kind: Kind::Alias { target: "id-ID" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
