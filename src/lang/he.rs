//! Hebrew (`he`): the bare-tag alias of [`he_il`](crate::he_il).
//!
//! `he` resolves to `he-IL`, currently the only Hebrew variant this crate
//! implements; it is not a claim that `he-IL` speaks for every
//! Hebrew-speaking region. Cardinal and ordinal are `he_il`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language has
//! no country to take a default currency from, so [`to_currency`] takes the
//! currency explicitly.
//!
//! ```
//! use n2words::{he, Currency};
//!
//! assert_eq!(he::to_cardinal(42)?, "ארבעים ושתים");
//! assert_eq!(he::to_currency(42.5, Currency::ILS)?, "ארבעים ושניים שקלים חמישים אגורות");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HE as CURRENCY_VOCAB;
use crate::he_il;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::he_il::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "he-IL";

options! {
    /// Options for [`to_currency_with`]: `he_il`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Hebrew's wording.
///
/// # Errors
///
/// As [`he_il::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{he::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::ILS);
/// assert_eq!(to_currency_with(1.01, &opts)?, "שקל אחד אגורה אחת");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`he_il::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    he_il::to_currency_with(
        value,
        &he_il::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `he`.
pub static LANGUAGE: Language = Language {
    code: "he",
    name: "Hebrew",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
