//! Hungarian (`hu`): the bare-tag alias of [`hu_hu`](crate::hu_hu).
//!
//! `hu` resolves to `hu-HU`, currently the only Hungarian variant this crate
//! implements. It exists so the language is reachable without a region
//! subtag; it is not a claim that `hu-HU` speaks for every Hungarian-speaking
//! region. Cardinal and ordinal are `hu_hu`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{hu, Currency};
//!
//! assert_eq!(hu::to_cardinal(42)?, "negyvenkettő");
//! assert_eq!(hu::to_ordinal(42)?, "negyvenkettődik");
//! assert_eq!(hu::to_currency(42.5, Currency::HUF)?, "negyvenkettő forint ötven fillér");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HU as CURRENCY_VOCAB;
use crate::hu_hu;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::hu_hu::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "hu-HU";

options! {
    /// Options for [`to_currency_with`]: `hu_hu`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Hungarian wording.
///
/// Unlike [`hu_hu::to_currency`], this needs an explicit currency: `hu` is a
/// language and has no country to take a default from.
///
/// # Errors
///
/// As [`hu_hu::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// # Errors
///
/// As [`hu_hu::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    hu_hu::to_currency_with(
        value,
        &hu_hu::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `hu`.
pub static LANGUAGE: Language = Language {
    code: "hu",
    name: "Hungarian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
