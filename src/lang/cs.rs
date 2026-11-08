//! Czech (`cs`): the bare-tag alias of [`cs_cz`](crate::cs_cz).
//!
//! `cs` resolves to `cs-CZ`, currently the only Czech variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `cs-CZ` speaks for every Czech-speaking
//! region. Cardinal and ordinal are `cs_cz`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{cs, Currency};
//!
//! assert_eq!(cs::to_cardinal(21)?, "dvacet jedna");
//! assert_eq!(cs::to_currency(42.5, Currency::CZK)?, "čtyřicet dva koruny padesát haléřů");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::cs_cz;
use crate::currency::Currency;
use crate::currency::vocab::CS as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::cs_cz::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "cs-CZ";

options! {
    /// Options for [`to_currency_with`]: `cs_cz`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Czech's default wording.
///
/// # Errors
///
/// As [`cs_cz::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{cs::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::CZK);
/// assert_eq!(to_currency_with(1, &opts)?, "jedna koruna");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`cs_cz::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    cs_cz::to_currency_with(
        value,
        &cs_cz::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `cs`.
pub static LANGUAGE: Language = Language {
    code: "cs",
    name: "Czech",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
