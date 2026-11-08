//! English (Singapore) (`en-SG`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-SG`.
pub static LANGUAGE: Language = Language {
    code: "en-SG",
    name: "English (Singapore)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
