//! Finnish (`fi`): the bare-tag alias of [`fi_fi`](crate::fi_fi).
//!
//! `fi` resolves to `fi-FI`, currently the only Finnish variant this crate
//! implements. It exists so the language is reachable without a region
//! subtag; it is not a claim that `fi-FI` speaks for every Finnish-speaking
//! region. Cardinal and ordinal are `fi_fi`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{fi, Currency};
//!
//! assert_eq!(fi::to_cardinal(42)?, "neljäkymmentäkaksi");
//! assert_eq!(fi::to_ordinal(42)?, "neljäkymmentätoinen");
//! assert_eq!(
//!     fi::to_currency(42.5, Currency::EUR)?,
//!     "neljäkymmentäkaksi euroa viisikymmentä senttiä"
//! );
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::FI as CURRENCY_VOCAB;
use crate::fi_fi;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::fi_fi::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "fi-FI";

options! {
    /// Options for [`to_currency_with`]: `fi_fi`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Finnish wording.
///
/// Unlike [`fi_fi::to_currency`], this needs an explicit currency: `fi` is a
/// language and has no country to take a default from.
///
/// # Errors
///
/// As [`fi_fi::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// # Errors
///
/// As [`fi_fi::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    fi_fi::to_currency_with(
        value,
        &fi_fi::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `fi`.
pub static LANGUAGE: Language = Language {
    code: "fi",
    name: "Finnish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
