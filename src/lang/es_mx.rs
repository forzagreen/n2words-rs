//! Mexican Spanish (`es-MX`): a locale profile of [`es_es`](crate::es_es).
//!
//! Numerals are identical to European Spanish (long-scale grammar, gender
//! agreement, "y" conjunction: the RAE and the Academia Mexicana agree on all
//! of it). "Which words" (Spanish) and "which country" (Mexico) are different
//! axes, and only the latter varies here: the default currency is MXN rather
//! than `es-ES`'s EUR. Any currency Spanish has words for is still reachable
//! through the `currency` option.
//!
//! ```
//! use n2words::es_mx::{to_cardinal, to_currency};
//!
//! assert_eq!(to_cardinal(21)?, "veintiuno");
//! assert_eq!(to_currency(42.5)?, "cuarenta y dos pesos con cincuenta centavos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::ES as CURRENCY_VOCAB;
use crate::es_es;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::es_es::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, OrdinalOptions, to_cardinal,
    to_cardinal_with, to_ordinal, to_ordinal_with,
};

/// The full implementation this profile delegates to.
pub const VARIANT_OF: &str = "es-ES";

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use "con" between the major and minor unit
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::MXN, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Mexican pesos.
///
/// ```
/// # use n2words::es_mx::to_currency;
/// assert_eq!(to_currency(1)?, "un peso");
/// assert_eq!(to_currency(1_000_000)?, "un millón de pesos");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`es_es::to_currency`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Spanish has words for.
///
/// # Errors
///
/// As [`es_es::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    es_es::to_currency_with(
        value,
        &es_es::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `es-MX`.
pub static LANGUAGE: Language = Language {
    code: "es-MX",
    name: "Mexican Spanish",
    kind: Kind::Profile { base: VARIANT_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
