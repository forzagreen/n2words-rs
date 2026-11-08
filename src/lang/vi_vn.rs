//! Vietnamese (Vietnam) (`vi-VN`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `vi-VN`.
pub static LANGUAGE: Language = Language {
    code: "vi-VN",
    name: "Vietnamese (Vietnam)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
