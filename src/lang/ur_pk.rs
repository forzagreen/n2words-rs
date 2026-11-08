//! Urdu (Pakistan) (`ur-PK`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ur-PK`.
pub static LANGUAGE: Language = Language {
    code: "ur-PK",
    name: "Urdu (Pakistan)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
