//! Greek (`el`): the bare-tag alias of [`el_gr`](crate::el_gr).
//!
//! `el` resolves to `el-GR`, currently the only Greek variant this crate
//! implements. It exists so the language is reachable without a region
//! subtag; it is not a claim that `el-GR` speaks for every Greek-speaking
//! region. Cardinal and ordinal are `el_gr`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{el, Currency};
//!
//! assert_eq!(el::to_cardinal(42)?, "σαράντα δύο");
//! assert_eq!(el::to_ordinal(42)?, "τεσσαρακοστός δεύτερος");
//! assert_eq!(el::to_currency(42.5, Currency::EUR)?, "σαράντα δύο ευρώ πενήντα λεπτά");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::EL as CURRENCY_VOCAB;
use crate::el_gr;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::el_gr::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "el-GR";

options! {
    /// Options for [`to_currency_with`]: `el_gr`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Greek wording.
///
/// Unlike [`el_gr::to_currency`], this needs an explicit currency: `el` is a
/// language and has no country to take a default from.
///
/// # Errors
///
/// As [`el_gr::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// # Errors
///
/// As [`el_gr::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    el_gr::to_currency_with(
        value,
        &el_gr::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `el`.
pub static LANGUAGE: Language = Language {
    code: "el",
    name: "Greek",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
