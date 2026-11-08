//! Georgian (`ka`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ka`.
pub static LANGUAGE: Language = Language {
    code: "ka",
    name: "Georgian",
    kind: Kind::Alias { target: "ka-GE" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
