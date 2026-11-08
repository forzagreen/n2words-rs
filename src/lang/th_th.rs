//! Thai (Thailand) (`th-TH`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `th-TH`.
pub static LANGUAGE: Language = Language {
    code: "th-TH",
    name: "Thai (Thailand)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
