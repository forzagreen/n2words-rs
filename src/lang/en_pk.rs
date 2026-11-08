//! English (Pakistan) (`en-PK`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-PK`.
pub static LANGUAGE: Language = Language {
    code: "en-PK",
    name: "English (Pakistan)",
    kind: Kind::Profile { base: "en-IN" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
