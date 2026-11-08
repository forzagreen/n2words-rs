//! Filipino (`fil`): the bare-tag alias of [`fil_ph`](crate::fil_ph).
//!
//! `fil` resolves to `fil-PH`, currently the only Filipino variant this crate
//! implements; it is not a claim that `fil-PH` speaks for every
//! Filipino-speaking region. Cardinal and ordinal are `fil_ph`'s own
//! functions. Currency is the one form a bare tag can't forward untouched: a
//! language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{fil, Currency};
//!
//! assert_eq!(fil::to_cardinal(256)?, "dalawang daang limampung anim");
//! assert_eq!(fil::to_ordinal(3)?, "ika-tatlo");
//! assert_eq!(fil::to_currency(42.5, Currency::PHP)?, "apatnapu dalawang piso at limampung sentimo");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::FIL as CURRENCY_VOCAB;
use crate::fil_ph;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::fil_ph::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "fil-PH";

options! {
    /// Options for [`to_currency_with`]: `fil_ph`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Filipino's wording.
///
/// # Errors
///
/// As [`fil_ph::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{fil::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::PHP);
/// assert_eq!(to_currency_with(0.5, &opts)?, "limampung sentimo");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`fil_ph::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    fil_ph::to_currency_with(
        value,
        &fil_ph::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `fil`.
pub static LANGUAGE: Language = Language {
    code: "fil",
    name: "Filipino",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
