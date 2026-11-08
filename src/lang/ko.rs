//! Korean (`ko`): the bare-tag alias of [`ko_kr`](crate::ko_kr).
//!
//! `ko` resolves to `ko-KR`, currently the only Korean variant the crate
//! implements. This entry point exists so the bare tag works without a region
//! subtag; it is not a claim that `ko-KR` speaks for every Korean-speaking
//! region. Cardinal and ordinal are `ko_kr`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ko, Currency};
//!
//! assert_eq!(ko::to_cardinal(21)?, "이십일");
//! assert_eq!(ko::to_currency(50000, Currency::KRW)?, "오만원");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::KO as CURRENCY_VOCAB;
use crate::ko_kr;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::ko_kr::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ko-KR";

options! {
    /// Options for [`to_currency_with`]: `ko_kr`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Korean's wording.
///
/// # Errors
///
/// As [`ko_kr::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ko::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::KRW);
/// assert_eq!(to_currency_with(-1000, &opts)?, "마이너스 천원");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ko_kr::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ko_kr::to_currency_with(
        value,
        &ko_kr::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ko`.
pub static LANGUAGE: Language = Language {
    code: "ko",
    name: "Korean",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
