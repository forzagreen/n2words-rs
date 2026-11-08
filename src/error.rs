//! The crate's error type.

use crate::currency::Currency;
use crate::language::FormKind;
use crate::max::Max;
use std::fmt;

/// A specialized [`Result`](std::result::Result) whose error is [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Why a conversion was refused.
///
/// Every form either returns a well-formed string or one of these; nothing
/// is ever silently truncated, clamped or guessed. The variants fall into two
/// families mirroring the JavaScript library's `TypeError` / `RangeError`
/// split: a *malformed* argument (unknown option, missing required option,
/// unparsable value) and a *well-formed but out-of-range* one (too large,
/// fractional ordinal, currency without a minor unit).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The input is not a finite decimal number (bad format, NaN, infinity).
    InvalidNumber(String),
    /// An ordinal input that is zero, negative or fractional.
    InvalidOrdinal(String),
    /// The value is at or past the form's declared ceiling.
    OutOfRange {
        /// The form's ceiling (the smallest value it refuses).
        max: Max,
    },
    /// A dynamic option name the form does not declare.
    UnknownOption {
        /// The offending key.
        key: String,
        /// The option names the form does declare (empty when it takes none).
        allowed: &'static [&'static str],
    },
    /// A required option (an alias's `currency`) was not supplied.
    MissingOption {
        /// The option's name.
        key: &'static str,
    },
    /// An option value that could not be parsed or is outside its allowed set.
    InvalidOptionValue {
        /// The option's name.
        key: &'static str,
        /// The value as supplied.
        value: String,
        /// The values the option accepts, when it has a fixed set.
        allowed: Vec<&'static str>,
    },
    /// The language has no words for this currency.
    UnsupportedCurrency {
        /// The currency that was asked for.
        currency: Currency,
        /// The currencies the language can name.
        allowed: Vec<Currency>,
    },
    /// A fractional amount in a currency with no minor unit (JPY, KRW, ...).
    NoMinorUnit {
        /// The currency.
        currency: Currency,
    },
    /// The language does not implement this form (dynamic API only).
    UnsupportedForm {
        /// The language's BCP 47 code.
        language: &'static str,
        /// The form that was asked for.
        form: FormKind,
    },
}

impl Error {
    /// Whether this error is about a *malformed* argument (the JavaScript
    /// library's `TypeError`) rather than a well-formed value out of range.
    #[must_use]
    pub fn is_malformed_argument(&self) -> bool {
        matches!(
            self,
            Error::UnknownOption { .. }
                | Error::MissingOption { .. }
                | Error::UnsupportedForm { .. }
        )
    }
}

fn join<T: fmt::Display>(items: impl IntoIterator<Item = T>) -> String {
    items
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidNumber(msg) | Error::InvalidOrdinal(msg) => f.write_str(msg),
            Error::OutOfRange { max } => write!(
                f,
                "Number too large to convert: the largest supported value is {}",
                max.largest_display()
            ),
            Error::UnknownOption { key, allowed: [] } => {
                write!(f, "Unknown option \"{key}\": this form takes no options")
            }
            Error::UnknownOption { key, allowed } => {
                write!(
                    f,
                    "Unknown option \"{key}\": expected one of: {}",
                    allowed.join(", ")
                )
            }
            Error::MissingOption { key } => write!(
                f,
                "Option \"{key}\" is required: a bare language tag names a language, not a locale, \
                 so it has no default to fall back on; pass it explicitly or use a region-qualified code"
            ),
            Error::InvalidOptionValue {
                key,
                value,
                allowed,
            } if allowed.is_empty() => {
                write!(f, "Option \"{key}\" has an invalid value: {value:?}")
            }
            Error::InvalidOptionValue {
                key,
                value,
                allowed,
            } => write!(
                f,
                "Option \"{key}\" must be one of: {}; got {value:?}",
                allowed.join(", ")
            ),
            Error::UnsupportedCurrency { currency, allowed } => write!(
                f,
                "Option \"currency\" must be one of: {}; got {currency}",
                join(allowed)
            ),
            Error::NoMinorUnit { currency } => {
                write!(
                    f,
                    "{currency} has no minor unit: fractional amounts aren't representable"
                )
            }
            Error::UnsupportedForm { language, form } => {
                write!(f, "{language} does not support the {form} form")
            }
        }
    }
}

impl std::error::Error for Error {}
