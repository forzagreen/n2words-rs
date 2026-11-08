//! Georgian (Georgia) (`ka-GE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ka-GE`.
pub static LANGUAGE: Language = Language {
    code: "ka-GE",
    name: "Georgian (Georgia)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
