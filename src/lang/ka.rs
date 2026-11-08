//! Georgian (`ka`): the bare-tag alias of [`ka_ge`](crate::ka_ge).
//!
//! `ka` resolves to `ka-GE`, currently the only Georgian variant this crate
//! implements. It exists so the language is reachable without a region
//! subtag; it is not a claim that `ka-GE` speaks for every Georgian-speaking
//! region. Cardinal and ordinal are `ka_ge`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ka, Currency};
//!
//! assert_eq!(ka::to_cardinal(42)?, "ორმოცდაორი");
//! assert_eq!(ka::to_ordinal(42)?, "მეორმოცდაორე");
//! assert_eq!(ka::to_currency(42.5, Currency::GEL)?, "ორმოცდაორი ლარი ორმოცდაათი თეთრი");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::KA as CURRENCY_VOCAB;
use crate::ka_ge;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::ka_ge::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ka-GE";

options! {
    /// Options for [`to_currency_with`]: `ka_ge`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Georgian wording.
///
/// Unlike [`ka_ge::to_currency`], this needs an explicit currency: `ka` is a
/// language and has no country to take a default from.
///
/// # Errors
///
/// As [`ka_ge::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// # Errors
///
/// As [`ka_ge::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ka_ge::to_currency_with(
        value,
        &ka_ge::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ka`.
pub static LANGUAGE: Language = Language {
    code: "ka",
    name: "Georgian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
