//! Hausa (Nigeria) (`ha-NG`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ha-NG`.
pub static LANGUAGE: Language = Language {
    code: "ha-NG",
    name: "Hausa (Nigeria)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
