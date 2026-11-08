//! Lithuanian (Lithuania) (`lt-LT`).
//!
//! **Not yet ported** — this is a placeholder so the registry compiles.

use crate::language::{Kind, Language};

/// Registry entry for `lt-LT`.
pub static LANGUAGE: Language = Language {
    code: "lt-LT",
    name: "Lithuanian (Lithuania)",
    kind: Kind::Full,
    cardinal: None,
    ordinal: None,
    currency: None,
};
