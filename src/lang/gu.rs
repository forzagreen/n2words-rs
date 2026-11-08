//! Gujarati (`gu`): the bare-tag alias of [`gu_in`](crate::gu_in).
//!
//! `gu` resolves to `gu-IN`, currently the only Gujarati variant the crate
//! implements; it is not a claim that `gu-IN` speaks for every
//! Gujarati-speaking region. Cardinal and ordinal are `gu_in`'s own
//! functions. Currency is the one form a bare tag can't forward untouched: a
//! language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{gu, Currency};
//!
//! assert_eq!(gu::to_cardinal(42)?, "બેતાળીસ");
//! assert_eq!(gu::to_currency(42.5, Currency::INR)?, "બેતાળીસ રૂપિયા પચાસ પૈસા");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::GU as CURRENCY_VOCAB;
use crate::gu_in;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::gu_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "gu-IN";

options! {
    /// Options for [`to_currency_with`]: `gu_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Gujarati's wording.
///
/// # Errors
///
/// As [`gu_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{gu::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1, &opts)?, "એક રૂપિયો");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`gu_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    gu_in::to_currency_with(
        value,
        &gu_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `gu`.
pub static LANGUAGE: Language = Language {
    code: "gu",
    name: "Gujarati",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
