//! French (`fr`): the bare-tag alias of [`fr_fr`](crate::fr_fr).
//!
//! `fr` has no single "correct" region, but unlike English, fr-FR and fr-BE
//! are currency-identical (both EUR euros/centimes) and diverge only in
//! cardinal grammar (septante/nonante vs. soixante-dix/quatre-vingt-dix).
//! This entry point defaults to fr-FR, the most widely used variant; it does
//! not speak for fr-BE or any other regional variant. Cardinal and ordinal are
//! `fr_fr`'s own functions. Currency is the one form a bare tag can't forward
//! untouched: a language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{fr, Currency};
//!
//! assert_eq!(fr::to_cardinal(80)?, "quatre-vingts");
//! assert_eq!(fr::to_ordinal(21)?, "vingt et unième");
//! assert_eq!(fr::to_currency(42.5, Currency::EUR)?, "quarante-deux euros et cinquante centimes");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::FR as CURRENCY_VOCAB;
use crate::fr_fr;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::fr_fr::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "fr-FR";

options! {
    /// Options for [`to_currency_with`]: `fr_fr`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "et" between euros and centimes
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with French's default wording.
///
/// # Errors
///
/// As [`fr_fr::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{fr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::EUR) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarante-deux euros cinquante centimes");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`fr_fr::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    fr_fr::to_currency_with(
        value,
        &fr_fr::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `fr`.
pub static LANGUAGE: Language = Language {
    code: "fr",
    name: "French",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
