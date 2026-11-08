//! Indonesian (`id`): the bare-tag alias of [`id_id`](crate::id_id).
//!
//! `id` resolves to `id-ID`, currently the only Indonesian variant this crate
//! implements; it is not a claim that `id-ID` speaks for every
//! Indonesian-speaking region. Cardinal and ordinal are `id_id`'s own
//! functions. Currency is the one form a bare tag can't forward untouched: a
//! language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{id, Currency};
//!
//! assert_eq!(id::to_cardinal(42)?, "empat puluh dua");
//! assert_eq!(id::to_ordinal(2)?, "kedua");
//! assert_eq!(id::to_currency(42, Currency::IDR)?, "empat puluh dua rupiah");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::ID as CURRENCY_VOCAB;
use crate::id_id;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::id_id::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "id-ID";

options! {
    /// Options for [`to_currency_with`]: `id_id`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Indonesian's wording.
///
/// # Errors
///
/// As [`id_id::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{id::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::IDR);
/// assert_eq!(to_currency_with(1000, &opts)?, "seribu rupiah");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`id_id::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    id_id::to_currency_with(
        value,
        &id_id::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `id`.
pub static LANGUAGE: Language = Language {
    code: "id",
    name: "Indonesian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
