//! Telugu (`te`): the bare-tag alias of [`te_in`](crate::te_in).
//!
//! `te` resolves to `te-IN`, currently the only Telugu variant this crate
//! implements. It exists purely so the bare tag works without a region
//! subtag; it is not a claim that `te-IN` speaks for every Telugu-speaking
//! region. Cardinal and ordinal are `te_in`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{te, Currency};
//!
//! assert_eq!(te::to_cardinal(1234)?, "ఒక వెయ్యి రెండు వందలు ముప్పై నాలుగు");
//! assert_eq!(te::to_currency(42.5, Currency::INR)?, "నలభై రెండు రూపాయలు యాభై పైసలు");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::TE as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::te_in;

pub use crate::te_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "te-IN";

options! {
    /// Options for [`to_currency_with`]: `te_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Telugu's wording.
///
/// # Errors
///
/// As [`te_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{te::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "ఒకటి రూపాయి ఒకటి పైసా");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`te_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    te_in::to_currency_with(
        value,
        &te_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `te`.
pub static LANGUAGE: Language = Language {
    code: "te",
    name: "Telugu",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
