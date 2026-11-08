//! Thai (`th`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `th`.
pub static LANGUAGE: Language = Language {
    code: "th",
    name: "Thai",
    kind: Kind::Alias { target: "th-TH" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
