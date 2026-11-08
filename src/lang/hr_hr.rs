//! Croatian (Croatia) (`hr-HR`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `hr-HR`.
pub static LANGUAGE: Language = Language {
    code: "hr-HR",
    name: "Croatian (Croatia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
