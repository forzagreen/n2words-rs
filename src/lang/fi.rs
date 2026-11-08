//! Finnish (`fi`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fi`.
pub static LANGUAGE: Language = Language {
    code: "fi",
    name: "Finnish",
    kind: Kind::Alias { target: "fi-FI" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
