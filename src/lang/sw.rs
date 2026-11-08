//! Swahili (`sw`): the bare-tag alias of [`sw_ke`](crate::sw_ke).
//!
//! `sw` resolves to `sw-KE`, currently the only Swahili variant this crate
//! implements. This entry point exists so the language can be addressed
//! without a region subtag; it is not a claim that `sw-KE` speaks for every
//! Swahili-speaking region. Cardinal and ordinal are `sw_ke`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{sw, Currency};
//!
//! assert_eq!(sw::to_cardinal(42)?, "arobaini na mbili");
//! assert_eq!(sw::to_ordinal(42)?, "wa arobaini na mbili");
//! assert_eq!(
//!     sw::to_currency(42.5, Currency::KES)?,
//!     "shilingi arobaini na mbili na senti hamsini"
//! );
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::SW as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::sw_ke;

pub use crate::sw_ke::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "sw-KE";

options! {
    /// Options for [`to_currency_with`]: `sw_ke`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Swahili's wording.
///
/// # Errors
///
/// As [`sw_ke::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{sw::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::KES);
/// assert_eq!(to_currency_with(1.5, &opts)?, "shilingi moja na senti hamsini");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`sw_ke::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    sw_ke::to_currency_with(
        value,
        &sw_ke::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `sw`.
pub static LANGUAGE: Language = Language {
    code: "sw",
    name: "Swahili",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
