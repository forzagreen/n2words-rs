//! Lithuanian (`lt`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `lt`.
pub static LANGUAGE: Language = Language {
    code: "lt",
    name: "Lithuanian",
    kind: Kind::Alias { target: "lt-LT" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
