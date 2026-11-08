//! Lithuanian (`lt`): the bare-tag alias of [`lt_lt`](crate::lt_lt).
//!
//! `lt` resolves to `lt-LT`, currently the only Lithuanian variant this crate
//! implements; it exists so the language can be named without a region
//! subtag, not as a claim that `lt-LT` speaks for every Lithuanian-speaking
//! region. Cardinal and ordinal are `lt_lt`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{lt, Currency};
//!
//! assert_eq!(lt::to_cardinal(42)?, "keturiasdešimt du");
//! assert_eq!(
//!     lt::to_currency(42.5, Currency::EUR)?,
//!     "keturiasdešimt du eurai penkiasdešimt centų"
//! );
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::LT as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::lt_lt;
use crate::number::Number;
use crate::options::options;

pub use crate::lt_lt::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "lt-LT";

options! {
    /// Options for [`to_currency_with`]: `lt_lt`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Lithuanian's default wording.
///
/// # Errors
///
/// As [`lt_lt::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{lt::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::EUR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "vienas euras vienas centas");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`lt_lt::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    lt_lt::to_currency_with(
        value,
        &lt_lt::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `lt`.
pub static LANGUAGE: Language = Language {
    code: "lt",
    name: "Lithuanian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
