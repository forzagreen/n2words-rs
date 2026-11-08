//! English (Singapore) (`en-SG`): a locale profile of [`en_gb`](crate::en_gb).
//!
//! Numerals are identical to British English (Commonwealth "and" after
//! hundreds, short scale); only the default currency differs (SGD). Any
//! currency English has words for is still reachable through the `currency`
//! option.
//!
//! ```
//! use n2words::en_sg::to_currency;
//!
//! assert_eq!(to_currency(42.5)?, "forty-two dollars and fifty cents");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::EN as CURRENCY_VOCAB;
use crate::en_gb;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::en_gb::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The full implementation this profile delegates to.
pub const VARIANT_OF: &str = "en-GB";

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use "and" between the major and minor unit
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::SGD, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Singapore dollars.
///
/// # Errors
///
/// As [`en_gb::to_currency`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency English has words for.
///
/// # Errors
///
/// As [`en_gb::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    en_gb::to_currency_with(
        value,
        &en_gb::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `en-SG`.
pub static LANGUAGE: Language = Language {
    code: "en-SG",
    name: "English (Singapore)",
    kind: Kind::Profile { base: VARIANT_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
