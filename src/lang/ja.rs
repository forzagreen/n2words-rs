//! Japanese (`ja`): the bare-tag alias of [`ja_jp`](crate::ja_jp).
//!
//! `ja` resolves to `ja-JP`, currently the only Japanese variant the crate
//! implements. This entry point exists so the bare tag works without a region
//! subtag; it is not a claim that `ja-JP` speaks for every Japanese-speaking
//! region. Cardinal and ordinal are `ja_jp`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ja, Currency};
//!
//! assert_eq!(ja::to_cardinal(42)?, "四十二");
//! assert_eq!(ja::to_currency(10000, Currency::JPY)?, "一万円");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::JA as CURRENCY_VOCAB;
use crate::ja_jp;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::ja_jp::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ja-JP";

options! {
    /// Options for [`to_currency_with`]: `ja_jp`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Japanese's wording.
///
/// # Errors
///
/// As [`ja_jp::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ja::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::JPY);
/// assert_eq!(to_currency_with(-1000, &opts)?, "マイナス千円");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ja_jp::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ja_jp::to_currency_with(
        value,
        &ja_jp::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ja`.
pub static LANGUAGE: Language = Language {
    code: "ja",
    name: "Japanese",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
