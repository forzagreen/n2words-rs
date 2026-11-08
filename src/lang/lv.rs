//! Latvian (`lv`): the bare-tag alias of [`lv_lv`](crate::lv_lv).
//!
//! `lv` resolves to `lv-LV`, currently the only Latvian variant this crate
//! implements; it exists so the language can be named without a region
//! subtag, not as a claim that `lv-LV` speaks for every Latvian-speaking
//! region. Cardinal and ordinal are `lv_lv`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{lv, Currency};
//!
//! assert_eq!(lv::to_cardinal(42)?, "četrdesmit divi");
//! assert_eq!(lv::to_currency(42.5, Currency::EUR)?, "četrdesmit divi eiro piecdesmit centu");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::LV as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::lv_lv;
use crate::number::Number;
use crate::options::options;

pub use crate::lv_lv::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "lv-LV";

options! {
    /// Options for [`to_currency_with`]: `lv_lv`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Latvian's default wording.
///
/// # Errors
///
/// As [`lv_lv::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{lv::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::EUR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "viens eiro viens cents");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`lv_lv::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    lv_lv::to_currency_with(
        value,
        &lv_lv::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `lv`.
pub static LANGUAGE: Language = Language {
    code: "lv",
    name: "Latvian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
