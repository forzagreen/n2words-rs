//! Swedish (`sv`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `sv`.
pub static LANGUAGE: Language = Language {
    code: "sv",
    name: "Swedish",
    kind: Kind::Alias { target: "sv-SE" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
