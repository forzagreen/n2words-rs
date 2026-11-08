//! Persian (`fa`): the bare-tag alias of [`fa_ir`](crate::fa_ir).
//!
//! `fa` resolves to `fa-IR`, currently the only Persian variant the crate
//! implements. This entry point exists purely so Persian is reachable without
//! a region subtag; it is not a claim that `fa-IR` speaks for every
//! Persian-speaking region. Cardinal and ordinal are `fa_ir`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{fa, Currency};
//!
//! assert_eq!(fa::to_cardinal(42)?, "چهل و دو");
//! assert_eq!(fa::to_currency(1000, Currency::IRR)?, "هزار ریال");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::FA as CURRENCY_VOCAB;
use crate::fa_ir;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::fa_ir::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "fa-IR";

options! {
    /// Options for [`to_currency_with`]: `fa_ir`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Persian's default wording.
///
/// ```
/// # use n2words::{fa::to_currency, Currency};
/// assert_eq!(to_currency(42, Currency::IRR)?, "چهل و دو ریال");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`fa_ir::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// # Errors
///
/// As [`fa_ir::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    fa_ir::to_currency_with(
        value,
        &fa_ir::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `fa`.
pub static LANGUAGE: Language = Language {
    code: "fa",
    name: "Persian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
