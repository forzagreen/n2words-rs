//! Croatian (`hr`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hr`.
pub static LANGUAGE: Language = Language {
    code: "hr",
    name: "Croatian",
    kind: Kind::Alias { target: "hr-HR" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
