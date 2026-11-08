//! Vietnamese (`vi`): the bare-tag alias of [`vi_vn`](crate::vi_vn).
//!
//! `vi` resolves to `vi-VN`, currently the only Vietnamese variant the crate
//! implements. This entry point exists so the bare tag works without a region
//! subtag; it is not a claim that `vi-VN` speaks for every Vietnamese-speaking
//! region. Cardinal and ordinal are `vi_vn`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{vi, Currency};
//!
//! assert_eq!(vi::to_cardinal(101)?, "một trăm lẻ một");
//! assert_eq!(vi::to_currency(21, Currency::VND)?, "hai mươi mốt đồng");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::VI as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::vi_vn;

pub use crate::vi_vn::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "vi-VN";

options! {
    /// Options for [`to_currency_with`]: `vi_vn`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Vietnamese's wording.
///
/// # Errors
///
/// As [`vi_vn::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{vi::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::VND);
/// assert_eq!(to_currency_with(-1000, &opts)?, "âm một nghìn đồng");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`vi_vn::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    vi_vn::to_currency_with(
        value,
        &vi_vn::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `vi`.
pub static LANGUAGE: Language = Language {
    code: "vi",
    name: "Vietnamese",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
