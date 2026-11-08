//! Norwegian Bokmål (`nb`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `nb`.
pub static LANGUAGE: Language = Language {
    code: "nb",
    name: "Norwegian Bokmål",
    kind: Kind::Alias { target: "nb-NO" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
