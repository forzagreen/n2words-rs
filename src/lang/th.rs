//! Thai (`th`): the bare-tag alias of [`th_th`](crate::th_th).
//!
//! `th` resolves to `th-TH`, currently the only Thai variant the crate
//! implements. This entry point exists so the bare tag works without a region
//! subtag; it is not a claim that `th-TH` speaks for every Thai-speaking
//! region. Cardinal and ordinal are `th_th`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{th, Currency};
//!
//! assert_eq!(th::to_cardinal(21)?, "ยี่สิบเอ็ด");
//! assert_eq!(th::to_currency(42.5, Currency::THB)?, "สี่สิบสองบาทห้าสิบสตางค์");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::TH as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::th_th;

pub use crate::th_th::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "th-TH";

options! {
    /// Options for [`to_currency_with`]: `th_th`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Thai's wording.
///
/// # Errors
///
/// As [`th_th::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{th::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::THB);
/// assert_eq!(to_currency_with(-1, &opts)?, "ลบหนึ่งบาทถ้วน");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`th_th::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    th_th::to_currency_with(
        value,
        &th_th::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `th`.
pub static LANGUAGE: Language = Language {
    code: "th",
    name: "Thai",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
