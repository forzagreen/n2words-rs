//! Swedish (`sv`): the bare-tag alias of [`sv_se`](crate::sv_se).
//!
//! `sv` resolves to `sv-SE`, currently the only Swedish variant this crate
//! implements. This entry point exists so the language can be addressed
//! without a region subtag; it is not a claim that `sv-SE` speaks for every
//! Swedish-speaking region. Cardinal and ordinal are `sv_se`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{sv, Currency};
//!
//! assert_eq!(sv::to_cardinal(42)?, "fyrtio-två");
//! assert_eq!(sv::to_ordinal(42)?, "fyrtio-tvåde");
//! assert_eq!(sv::to_currency(42.5, Currency::SEK)?, "fyrtio-två kronor och femtio öre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::SV as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::sv_se;

pub use crate::sv_se::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "sv-SE";

options! {
    /// Options for [`to_currency_with`]: `sv_se`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Swedish's wording.
///
/// # Errors
///
/// As [`sv_se::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{sv::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::SEK);
/// assert_eq!(to_currency_with(1.5, &opts)?, "en krona och femtio öre");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`sv_se::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    sv_se::to_currency_with(
        value,
        &sv_se::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `sv`.
pub static LANGUAGE: Language = Language {
    code: "sv",
    name: "Swedish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
