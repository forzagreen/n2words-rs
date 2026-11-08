//! Azerbaijani (`az`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `az`.
pub static LANGUAGE: Language = Language {
    code: "az",
    name: "Azerbaijani",
    kind: Kind::Alias { target: "az-AZ" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
