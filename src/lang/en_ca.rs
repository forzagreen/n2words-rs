//! Canadian English (`en-CA`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-CA`.
pub static LANGUAGE: Language = Language {
    code: "en-CA",
    name: "Canadian English",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
