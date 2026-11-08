//! Tamil (`ta`): the bare-tag alias of [`ta_in`](crate::ta_in).
//!
//! `ta` resolves to `ta-IN`, currently the only Tamil variant this crate
//! implements. It exists purely so the bare tag works without a region
//! subtag; it is not a claim that `ta-IN` speaks for every Tamil-speaking
//! region. Cardinal and ordinal are `ta_in`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ta, Currency};
//!
//! assert_eq!(ta::to_cardinal(1234)?, "ஒரு ஆயிரம் இருநூற்று முப்பத்திநான்கு");
//! assert_eq!(ta::to_currency(42.5, Currency::INR)?, "நாற்பத்திரண்டு ரூபாய் ஐம்பது பைசா");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::TA as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::ta_in;

pub use crate::ta_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ta-IN";

options! {
    /// Options for [`to_currency_with`]: `ta_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Tamil's wording.
///
/// # Errors
///
/// As [`ta_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ta::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "ஒன்று ரூபாய் ஒன்று பைசா");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ta_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ta_in::to_currency_with(
        value,
        &ta_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ta`.
pub static LANGUAGE: Language = Language {
    code: "ta",
    name: "Tamil",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
