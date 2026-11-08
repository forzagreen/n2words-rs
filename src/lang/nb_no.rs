//! Norwegian Bokmål (Norway) (`nb-NO`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `nb-NO`.
pub static LANGUAGE: Language = Language {
    code: "nb-NO",
    name: "Norwegian Bokmål (Norway)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
