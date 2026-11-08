//! Croatian (`hr`): the bare-tag alias of [`hr_hr`](crate::hr_hr).
//!
//! `hr` resolves to `hr-HR`, currently the only Croatian variant this crate
//! implements; it exists so the language can be named without a region
//! subtag, not as a claim that `hr-HR` speaks for every Croatian-speaking
//! region. Cardinal and ordinal are `hr_hr`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{hr, Currency};
//!
//! assert_eq!(hr::to_cardinal(42)?, "četrdeset dva");
//! assert_eq!(hr::to_currency(42.5, Currency::EUR)?, "četrdeset dva eura pedeset centi");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HR as CURRENCY_VOCAB;
use crate::hr_hr;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::hr_hr::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "hr-HR";

options! {
    /// Options for [`to_currency_with`]: `hr_hr`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Croatian's default wording.
///
/// # Errors
///
/// As [`hr_hr::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{hr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::EUR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "jedan euro jedan cent");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`hr_hr::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    hr_hr::to_currency_with(
        value,
        &hr_hr::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `hr`.
pub static LANGUAGE: Language = Language {
    code: "hr",
    name: "Croatian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
