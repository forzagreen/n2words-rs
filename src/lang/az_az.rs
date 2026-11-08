//! Azerbaijani (Azerbaijan) (`az-AZ`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `az-AZ`.
pub static LANGUAGE: Language = Language {
    code: "az-AZ",
    name: "Azerbaijani (Azerbaijan)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
