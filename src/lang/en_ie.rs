//! English (Ireland) (`en-IE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `en-IE`.
pub static LANGUAGE: Language = Language {
    code: "en-IE",
    name: "English (Ireland)",
    kind: Kind::Profile { base: "en-GB" },
    cardinal: None,
    ordinal: None,
    currency: None,
};
