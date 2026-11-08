//! English (Nigeria) (`en-NG`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-NG`.
pub static LANGUAGE: Language = Language {
    code: "en-NG",
    name: "English (Nigeria)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
