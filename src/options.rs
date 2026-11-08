//! Per-form options: typed structs for the Rust API, plus the declarative
//! metadata behind the dynamic API, the CLI and the generated docs.
//!
//! A form that takes options declares them once with the `options!` macro.
//! That declaration yields the public struct (with its `Default`), the
//! [`OptionSpec`] list describing each option, and the parser that builds the
//! struct from `(name, value)` string pairs, so a typo in either direction
//! fails to compile instead of drifting.

use crate::currency::CurrencyTable;
use crate::error::{Error, Result};
use std::fmt;

/// What kind of value an option takes.
#[derive(Debug, Clone, Copy)]
pub enum OptionKind {
    /// `true` / `false`.
    Bool,
    /// Free text (a custom negative or conjunction word, say).
    Text,
    /// One of a fixed set of words.
    Enum(&'static [&'static str]),
    /// An ISO 4217 code the language has words for.
    Currency(&'static CurrencyTable),
}

/// One option a form declares.
#[derive(Debug, Clone, Copy)]
pub struct OptionSpec {
    /// The option's name, in snake_case (`hundred_pairing`).
    pub name: &'static str,
    /// The kind of value it takes.
    pub kind: OptionKind,
    /// Renders the default value, or `None` when the option is required.
    pub default: Option<fn() -> String>,
    /// A one-line description, from the doc comment on the field.
    pub description: &'static str,
}

impl OptionSpec {
    /// The allowed values, when the option has a fixed set.
    #[must_use]
    pub fn values(&self) -> Option<Vec<&'static str>> {
        match self.kind {
            OptionKind::Bool => Some(vec!["true", "false"]),
            OptionKind::Text => None,
            OptionKind::Enum(values) => Some(values.to_vec()),
            OptionKind::Currency(table) => Some(table.codes().map(|c| c.code()).collect()),
        }
    }

    /// The default value rendered as text, or `None` when the option is required.
    #[must_use]
    pub fn default_value(&self) -> Option<String> {
        self.default.map(|render| render())
    }

    /// Whether the option must be supplied (it has no default).
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.default.is_none()
    }

    /// The description with the doc comment's leading space trimmed.
    #[must_use]
    pub fn description(&self) -> &'static str {
        self.description.trim()
    }

    /// The type as the docs render it: `boolean`, `string`, or the enum set.
    #[must_use]
    pub fn type_display(&self) -> String {
        match self.kind {
            OptionKind::Bool => "boolean".into(),
            OptionKind::Text => "string".into(),
            OptionKind::Enum(values) => values
                .iter()
                .map(|v| format!("'{v}'"))
                .collect::<Vec<_>>()
                .join(" | "),
            OptionKind::Currency(table) => table
                .codes()
                .map(|c| format!("'{}'", c.code()))
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }
}

/// A value type an option field may have.
pub trait OptionValue: Sized {
    /// The kind of value, for the dynamic API and docs.
    const KIND: OptionKind;
    /// Parses the value from text, using `spec` to name the option and its
    /// allowed set in errors.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidOptionValue`] when `raw` is not a valid value.
    fn parse(spec: &OptionSpec, raw: &str) -> Result<Self>;
    /// Renders the value as text, the inverse of [`parse`](Self::parse).
    fn render(&self) -> String;
}

impl OptionValue for bool {
    const KIND: OptionKind = OptionKind::Bool;

    fn parse(spec: &OptionSpec, raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(Error::InvalidOptionValue {
                key: spec.name,
                value: raw.to_string(),
                allowed: vec!["true", "false"],
            }),
        }
    }

    fn render(&self) -> String {
        self.to_string()
    }
}

impl OptionValue for String {
    const KIND: OptionKind = OptionKind::Text;

    fn parse(_spec: &OptionSpec, raw: &str) -> Result<Self> {
        Ok(raw.to_string())
    }

    fn render(&self) -> String {
        self.clone()
    }
}

/// Rejects any dynamic option for a form that declares none.
///
/// # Errors
///
/// [`Error::UnknownOption`] naming the first key supplied.
pub fn ensure_no_options(options: &[(&str, &str)]) -> Result<()> {
    match options.first() {
        None => Ok(()),
        Some((key, _)) => Err(Error::UnknownOption {
            key: (*key).to_string(),
            allowed: &[],
        }),
    }
}

/// The error of an option enum's `FromStr`: the text named no variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseValueError;

impl fmt::Display for ParseValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no such value")
    }
}

impl std::error::Error for ParseValueError {}

/// Declares an enum-valued option type: the enum, its allowed words, `FromStr`,
/// `Display` and the [`OptionValue`] glue.
///
/// ```ignore
/// option_enum! {
///     /// Grammatical gender.
///     pub enum Gender { Masculine = "masculine", Feminine = "feminine" }
/// }
/// ```
macro_rules! option_enum {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $text:literal ),+ $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $( $(#[$vmeta])* $variant, )+
        }

        impl $name {
            /// Every value, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            /// Every value's text form, in declaration order.
            pub const VALUES: &'static [&'static str] = &[$($text),+];

            /// The value's text form (`"masculine"`).
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::options::ParseValueError;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    _ => Err($crate::options::ParseValueError),
                }
            }
        }

        impl $crate::options::OptionValue for $name {
            const KIND: $crate::options::OptionKind = $crate::options::OptionKind::Enum(Self::VALUES);

            fn parse(spec: &$crate::options::OptionSpec, raw: &str) -> $crate::Result<Self> {
                raw.parse().map_err(|_| $crate::Error::InvalidOptionValue {
                    key: spec.name,
                    value: raw.to_string(),
                    allowed: Self::VALUES.to_vec(),
                })
            }

            fn render(&self) -> String {
                self.as_str().to_string()
            }
        }
    };
}
pub(crate) use option_enum;

/// Picks the [`OptionKind`] for a field: a currency table when one is named,
/// otherwise the type's own kind.
macro_rules! option_kind {
    ($ty:ty, $table:expr) => {
        $crate::options::OptionKind::Currency(&$table)
    };
    ($ty:ty) => {
        <$ty as $crate::options::OptionValue>::KIND
    };
}
pub(crate) use option_kind;

/// Declares a form's options struct together with its metadata and parser.
///
/// Every field needs a doc comment (it becomes the option's description), a
/// type implementing [`OptionValue`], and a default. A `currency` field names
/// the language's vocabulary table with `table = ...` so the allowed set is
/// derived from it. A bare-tag alias marks its `currency` as `required`
/// (first, before any doc comment); the struct then has no `Default` but a
/// `new(currency)` constructor that fills every other option with its default.
///
/// ```ignore
/// options! {
///     /// Options for [`to_cardinal_with`].
///     pub struct CardinalOptions {
///         /// Use "and" after hundreds.
///         and: bool = false;
///         /// Grammatical gender of the counted noun.
///         gender: Gender = Gender::Masculine;
///     }
/// }
/// ```
macro_rules! options {
    // A struct with one required field (an alias's currency).
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            required
            $(#[doc = $rdoc:expr])*
            $rfield:ident : $rty:ty $(, table = $rtable:expr)? ;
            $(
                $(#[doc = $doc:expr])*
                $field:ident : $ty:ty = $default:expr $(, table = $table:expr)? ;
            )*
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, PartialEq)]
        $vis struct $name {
            $(#[doc = $rdoc])*
            pub $rfield: $rty,
            $(
                $(#[doc = $doc])*
                pub $field: $ty,
            )*
        }

        impl $name {
            /// The options with the required value set and every other option
            /// at its default.
            #[must_use]
            pub fn new($rfield: $rty) -> Self {
                Self { $rfield, $($field: $default,)* }
            }

            /// The option names this form declares.
            pub const NAMES: &'static [&'static str] = &[stringify!($rfield), $(stringify!($field)),*];

            /// The options this form declares, in declaration order.
            pub const SPEC: &'static [$crate::options::OptionSpec] = &[
                $crate::options::OptionSpec {
                    name: stringify!($rfield),
                    kind: $crate::options::option_kind!($rty $(, $rtable)?),
                    default: None,
                    description: concat!("" $(, $rdoc)*),
                },
                $(
                    $crate::options::OptionSpec {
                        name: stringify!($field),
                        kind: $crate::options::option_kind!($ty $(, $table)?),
                        default: Some(|| <$ty as $crate::options::OptionValue>::render(&$default)),
                        description: concat!("" $(, $doc)*),
                    },
                )*
            ];

            /// Builds the options from dynamic `(name, value)` pairs, as the
            /// CLI and the registry's dynamic API do.
            ///
            /// # Errors
            ///
            /// [`Error::UnknownOption`](crate::Error::UnknownOption) for a name
            /// not declared here, [`Error::MissingOption`](crate::Error::MissingOption)
            /// when the required option is absent, and
            /// [`Error::InvalidOptionValue`](crate::Error::InvalidOptionValue)
            /// for a value that does not parse.
            pub fn from_pairs(pairs: &[(&str, &str)]) -> $crate::Result<Self> {
                let mut $rfield: Option<$rty> = None;
                $( let mut $field: $ty = $default; )*
                for (key, raw) in pairs {
                    match *key {
                        stringify!($rfield) => {
                            $rfield = Some(<$rty as $crate::options::OptionValue>::parse(&Self::SPEC[0], raw)?);
                        }
                        $(
                            stringify!($field) => {
                                let spec = Self::SPEC.iter().find(|s| s.name == stringify!($field)).expect("declared");
                                $field = <$ty as $crate::options::OptionValue>::parse(spec, raw)?;
                            }
                        )*
                        _ => return Err($crate::Error::UnknownOption { key: (*key).to_string(), allowed: Self::NAMES }),
                    }
                }
                let $rfield = $rfield.ok_or($crate::Error::MissingOption { key: stringify!($rfield) })?;
                Ok(Self { $rfield, $($field,)* })
            }
        }
    };
    // A struct where every field has a default.
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[doc = $doc:expr])*
                $field:ident : $ty:ty = $default:expr $(, table = $table:expr)? ;
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, PartialEq)]
        $vis struct $name {
            $(
                $(#[doc = $doc])*
                pub $field: $ty,
            )+
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $default,)+ }
            }
        }

        impl $name {
            /// The option names this form declares.
            pub const NAMES: &'static [&'static str] = &[$(stringify!($field)),+];

            /// The options this form declares, in declaration order.
            pub const SPEC: &'static [$crate::options::OptionSpec] = &[
                $(
                    $crate::options::OptionSpec {
                        name: stringify!($field),
                        kind: $crate::options::option_kind!($ty $(, $table)?),
                        default: Some(|| <$ty as $crate::options::OptionValue>::render(&$default)),
                        description: concat!("" $(, $doc)*),
                    },
                )+
            ];

            /// Builds the options from dynamic `(name, value)` pairs, as the
            /// CLI and the registry's dynamic API do.
            ///
            /// # Errors
            ///
            /// [`Error::UnknownOption`](crate::Error::UnknownOption) for a name
            /// not declared here, and
            /// [`Error::InvalidOptionValue`](crate::Error::InvalidOptionValue)
            /// for a value that does not parse.
            pub fn from_pairs(pairs: &[(&str, &str)]) -> $crate::Result<Self> {
                let mut out = Self::default();
                for (key, raw) in pairs {
                    match *key {
                        $(
                            stringify!($field) => {
                                let spec = Self::SPEC.iter().find(|s| s.name == stringify!($field)).expect("declared");
                                out.$field = <$ty as $crate::options::OptionValue>::parse(spec, raw)?;
                            }
                        )+
                        _ => return Err($crate::Error::UnknownOption { key: (*key).to_string(), allowed: Self::NAMES }),
                    }
                }
                Ok(out)
            }
        }
    };
}
pub(crate) use options;

option_enum! {
    /// Grammatical gender, for languages whose number words agree with a noun.
    pub enum Gender {
        /// Masculine agreement.
        Masculine = "masculine",
        /// Feminine agreement.
        Feminine = "feminine",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;

    options! {
        /// Test options.
        pub struct Sample {
            /// A flag.
            flag: bool = false;
            /// A gender.
            gender: Gender = Gender::Masculine;
            /// A word.
            word: String = "x".to_string();
            /// A currency.
            currency: Currency = Currency::USD, table = crate::currency::vocab::EN;
        }
    }

    options! {
        /// Alias options.
        pub struct AliasSample {
            required
            /// The currency.
            currency: Currency, table = crate::currency::vocab::FR;
            /// A flag.
            and: bool = true;
        }
    }

    #[test]
    fn defaults_and_spec() {
        let d = Sample::default();
        assert_eq!(
            d,
            Sample {
                flag: false,
                gender: Gender::Masculine,
                word: "x".into(),
                currency: Currency::USD
            }
        );
        assert_eq!(Sample::NAMES, &["flag", "gender", "word", "currency"]);
        assert_eq!(Sample::SPEC[0].default_value().as_deref(), Some("false"));
        assert_eq!(
            Sample::SPEC[1].default_value().as_deref(),
            Some("masculine")
        );
        assert_eq!(
            Sample::SPEC[1].values(),
            Some(vec!["masculine", "feminine"])
        );
        assert_eq!(Sample::SPEC[2].description(), "A word.");
        assert!(Sample::SPEC[3].values().unwrap().contains(&"GBP"));
    }

    #[test]
    fn from_pairs_parses_and_validates() {
        let s = Sample::from_pairs(&[
            ("flag", "true"),
            ("gender", "feminine"),
            ("currency", "gbp"),
        ])
        .unwrap();
        assert_eq!(
            s,
            Sample {
                flag: true,
                gender: Gender::Feminine,
                word: "x".into(),
                currency: Currency::GBP
            }
        );
        assert!(matches!(
            Sample::from_pairs(&[("nope", "1")]),
            Err(Error::UnknownOption { .. })
        ));
        assert!(matches!(
            Sample::from_pairs(&[("flag", "maybe")]),
            Err(Error::InvalidOptionValue { key: "flag", .. })
        ));
        assert!(matches!(
            Sample::from_pairs(&[("gender", "neuter")]),
            Err(Error::InvalidOptionValue { key: "gender", .. })
        ));
        assert!(matches!(
            Sample::from_pairs(&[("currency", "XXX")]),
            Err(Error::InvalidOptionValue {
                key: "currency",
                ..
            })
        ));
        // A real code the language has no words for is reported with the allowed set.
        assert!(matches!(
            Sample::from_pairs(&[("currency", "JPY")]),
            Err(Error::UnsupportedCurrency {
                currency: Currency::JPY,
                ..
            })
        ));
    }

    #[test]
    fn required_field() {
        assert!(matches!(
            AliasSample::from_pairs(&[]),
            Err(Error::MissingOption { key: "currency" })
        ));
        let a = AliasSample::from_pairs(&[("currency", "EUR"), ("and", "false")]).unwrap();
        assert_eq!(
            a,
            AliasSample {
                currency: Currency::EUR,
                and: false
            }
        );
        assert_eq!(
            AliasSample::new(Currency::MAD),
            AliasSample {
                currency: Currency::MAD,
                and: true
            }
        );
        assert!(AliasSample::SPEC[0].is_required());
        assert_eq!(AliasSample::SPEC[0].default_value(), None);
    }

    #[test]
    fn no_options() {
        assert!(ensure_no_options(&[]).is_ok());
        assert!(matches!(
            ensure_no_options(&[("x", "1")]),
            Err(Error::UnknownOption { allowed: &[], .. })
        ));
    }
}
