//! Biblical Hebrew (`hbo`): the bare-tag alias of [`hbo_il`](crate::hbo_il).
//!
//! `hbo` resolves to `hbo-IL`, currently the only Biblical Hebrew variant this
//! crate implements. Cardinal and ordinal are `hbo_il`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language has
//! no country to take a default currency from, so [`to_currency`] takes the
//! currency explicitly.
//!
//! ```
//! use n2words::{hbo, Currency};
//!
//! assert_eq!(hbo::to_cardinal(42)?, "ארבעים ושניים");
//! assert_eq!(hbo::to_currency(42.5, Currency::ILS)?, "ארבעים ושניים שקלים חמישים גרות");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HBO as CURRENCY_VOCAB;
use crate::hbo_il;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::hbo_il::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "hbo-IL";

options! {
    /// Options for [`to_currency_with`]: `hbo_il`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Biblical Hebrew's wording.
///
/// # Errors
///
/// As [`hbo_il::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{hbo::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::ILS);
/// assert_eq!(to_currency_with(1.01, &opts)?, "שקל אחד גרה אחת");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`hbo_il::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    hbo_il::to_currency_with(
        value,
        &hbo_il::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `hbo`.
pub static LANGUAGE: Language = Language {
    code: "hbo",
    name: "Biblical Hebrew",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
