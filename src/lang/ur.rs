//! Urdu (`ur`): the bare-tag alias of [`ur_pk`](crate::ur_pk).
//!
//! `ur` resolves to `ur-PK`, currently the only Urdu variant this crate
//! implements. It exists purely so the bare tag works without a region
//! subtag; it is not a claim that `ur-PK` speaks for every Urdu-speaking
//! region. Cardinal and ordinal are `ur_pk`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ur, Currency};
//!
//! assert_eq!(ur::to_cardinal(215)?, "دو سو پندرہ");
//! assert_eq!(ur::to_currency(42.5, Currency::PKR)?, "بیالیس روپے پچاس پیسے");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::UR as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::ur_pk;

pub use crate::ur_pk::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ur-PK";

options! {
    /// Options for [`to_currency_with`]: `ur_pk`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Urdu's wording.
///
/// # Errors
///
/// As [`ur_pk::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ur::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::PKR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "ایک روپیہ ایک پیسہ");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ur_pk::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ur_pk::to_currency_with(
        value,
        &ur_pk::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ur`.
pub static LANGUAGE: Language = Language {
    code: "ur",
    name: "Urdu",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
