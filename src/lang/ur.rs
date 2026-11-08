//! Urdu (`ur`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `ur`.
pub static LANGUAGE: Language = Language {
    code: "ur",
    name: "Urdu",
    kind: Kind::Alias { target: "ur-PK" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
