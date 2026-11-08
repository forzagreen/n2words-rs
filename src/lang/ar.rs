//! Arabic (`ar`): the bare-tag alias of [`ar_sa`](crate::ar_sa).
//!
//! `ar` resolves to `ar-SA`, currently the only Arabic variant the crate
//! implements. This entry point exists purely so Arabic is reachable without
//! a region subtag; it is not a claim that `ar-SA` speaks for every
//! Arabic-speaking region. Cardinal and ordinal are `ar_sa`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{ar, Currency};
//!
//! assert_eq!(ar::to_cardinal(42)?, "اثنان وأربعون");
//! assert_eq!(ar::to_currency(42.5, Currency::SAR)?, "اثنان وأربعون ريالاً وخمسون هللة");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::ar_sa;
use crate::currency::Currency;
use crate::currency::vocab::AR as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::ar_sa::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, Case, HundredSpelling, ORDINAL_MAX,
    OrdinalOptions, to_cardinal, to_cardinal_with, to_ordinal, to_ordinal_with,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ar-SA";

options! {
    /// Options for [`to_currency_with`]: `ar_sa`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Grammatical case (الإعراب): nominative/مرفوع (ريالان) or, as after مبلغ, accusative/منصوب or genitive/مجرور (ريالين)
        case: Case = Case::Nominative;
        /// Spelling of 100: the academy-standard مئة or the traditional مائة, as on banknotes
        hundred_spelling: HundredSpelling = HundredSpelling::Mia;
    }
}

/// Spells an amount in the given currency, with Arabic's default wording.
///
/// ```
/// # use n2words::{ar::to_currency, Currency};
/// assert_eq!(to_currency(1, Currency::EGP)?, "جنيه واحد");
/// assert_eq!(to_currency("1.500", Currency::TND)?, "دينار واحد وخمسمئة مليم");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ar_sa::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ar::{to_currency_with, Case, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { case: Case::Genitive, ..CurrencyOptions::new(Currency::BHD) };
/// assert_eq!(to_currency_with(2, &opts)?, "دينارين");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ar_sa::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ar_sa::to_currency_with(
        value,
        &ar_sa::CurrencyOptions {
            currency: options.currency,
            case: options.case,
            hundred_spelling: options.hundred_spelling,
        },
    )
}

/// Registry entry for `ar`.
pub static LANGUAGE: Language = Language {
    code: "ar",
    name: "Arabic",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
