//! English (`en`): the bare-tag alias of [`en_us`](crate::en_us).
//!
//! English has no single "correct" region, so this entry point defaults to
//! American English, the most widely used variant. Cardinal and ordinal are
//! `en_us`'s own functions. Currency is the one form a bare tag can't forward
//! untouched: a language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{en, Currency};
//!
//! assert_eq!(en::to_cardinal(42)?, "forty-two");
//! assert_eq!(en::to_currency(42.5, Currency::GBP)?, "forty-two pounds and fifty pence");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::EN as CURRENCY_VOCAB;
use crate::en_us;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::en_us::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "en-US";

options! {
    /// Options for [`to_currency_with`]: `en_us`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "and" between the major and minor unit (e.g., "one dollar and fifty cents")
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with English's default wording.
///
/// # Errors
///
/// As [`en_us::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{en::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::USD) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "forty-two dollars fifty cents");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`en_us::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    en_us::to_currency_with(
        value,
        &en_us::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `en`.
pub static LANGUAGE: Language = Language {
    code: "en",
    name: "English",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
