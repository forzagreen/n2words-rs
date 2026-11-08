//! French (Belgium) (`fr-BE`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `fr-BE`.
pub static LANGUAGE: Language = Language {
    code: "fr-BE",
    name: "French (Belgium)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
