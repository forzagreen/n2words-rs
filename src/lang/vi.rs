//! Vietnamese (`vi`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `vi`.
pub static LANGUAGE: Language = Language {
    code: "vi",
    name: "Vietnamese",
    kind: Kind::Alias { target: "vi-VN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
