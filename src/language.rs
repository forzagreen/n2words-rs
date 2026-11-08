//! The language registry: every entry point, its forms, their ceilings and
//! options, reachable by BCP 47 code at runtime.
//!
//! The typed API (`n2words::en::to_cardinal(42)`) is the primary one. This
//! dynamic layer exists for the CLI and for callers that pick a language from
//! data; it wraps the same functions, so the two can't disagree.

use crate::error::{Error, Result};
use crate::max::Max;
use crate::number::Number;
use crate::options::OptionSpec;
use std::fmt;

/// The three things a language can spell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormKind {
    /// "forty-two".
    Cardinal,
    /// "forty-second".
    Ordinal,
    /// "forty-two dollars and fifty cents".
    Currency,
}

impl FormKind {
    /// All three forms, in the order the docs list them.
    pub const ALL: [FormKind; 3] = [FormKind::Cardinal, FormKind::Ordinal, FormKind::Currency];

    /// The form's lowercase name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            FormKind::Cardinal => "cardinal",
            FormKind::Ordinal => "ordinal",
            FormKind::Currency => "currency",
        }
    }
}

impl fmt::Display for FormKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for FormKind {
    type Err = crate::options::ParseValueError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "cardinal" => Ok(FormKind::Cardinal),
            "ordinal" => Ok(FormKind::Ordinal),
            "currency" => Ok(FormKind::Currency),
            _ => Err(crate::options::ParseValueError),
        }
    }
}

/// What kind of entry point a language module is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A full implementation owning its numerals.
    Full,
    /// A locale profile: numerals identical to `base`, differing only in the
    /// default currency.
    Profile {
        /// The full implementation the profile delegates to.
        base: &'static str,
    },
    /// A bare-tag alias (`en` for `en-US`): the target's forms, except that
    /// currency must be named explicitly since a language has no country.
    Alias {
        /// The variant the alias resolves to.
        target: &'static str,
    },
}

/// The dynamic entry point of one form: a value and `(name, value)` option
/// pairs in, words out.
pub type ConvertFn = fn(Number<'_>, &[(&str, &str)]) -> Result<String>;

/// One form of one language, as the registry sees it.
#[derive(Debug, Clone, Copy)]
pub struct Form {
    /// The form's ceiling.
    pub max: Max,
    /// The options the form declares (empty when it takes none).
    pub options: &'static [OptionSpec],
    /// The conversion itself.
    pub convert: ConvertFn,
}

impl Form {
    /// Converts `value` with dynamic options.
    ///
    /// # Errors
    ///
    /// Whatever the typed function returns, plus the option-parsing errors
    /// ([`Error::UnknownOption`], [`Error::MissingOption`],
    /// [`Error::InvalidOptionValue`]).
    pub fn convert<'a>(
        &self,
        value: impl Into<Number<'a>>,
        options: &[(&str, &str)],
    ) -> Result<String> {
        (self.convert)(value.into(), options)
    }

    /// The spec of the option called `name`, if the form declares it.
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&'static OptionSpec> {
        self.options.iter().find(|o| o.name == name)
    }
}

/// A language entry point: its code, its kind and the forms it implements.
#[derive(Debug)]
pub struct Language {
    /// The canonical BCP 47 code (`"en-US"`, `"zh-Hans-CN"`, `"de"`).
    pub code: &'static str,
    /// The English display name (`"American English"`).
    pub name: &'static str,
    /// Full implementation, locale profile or bare-tag alias.
    pub kind: Kind,
    /// The cardinal form, if implemented.
    pub cardinal: Option<Form>,
    /// The ordinal form, if implemented.
    pub ordinal: Option<Form>,
    /// The currency form, if implemented.
    pub currency: Option<Form>,
}

impl Language {
    /// The form of the given kind, if implemented.
    #[must_use]
    pub fn form(&self, kind: FormKind) -> Option<&Form> {
        match kind {
            FormKind::Cardinal => self.cardinal.as_ref(),
            FormKind::Ordinal => self.ordinal.as_ref(),
            FormKind::Currency => self.currency.as_ref(),
        }
    }

    /// The forms the language implements, in [`FormKind::ALL`] order.
    pub fn forms(&self) -> impl Iterator<Item = (FormKind, &Form)> {
        FormKind::ALL
            .into_iter()
            .filter_map(|k| self.form(k).map(|f| (k, f)))
    }

    /// Converts `value` in the given form with dynamic options.
    ///
    /// # Errors
    ///
    /// [`Error::UnsupportedForm`] when the language lacks the form, otherwise
    /// whatever [`Form::convert`] returns.
    pub fn convert<'a>(
        &self,
        kind: FormKind,
        value: impl Into<Number<'a>>,
        options: &[(&str, &str)],
    ) -> Result<String> {
        let form = self.form(kind).ok_or(Error::UnsupportedForm {
            language: self.code,
            form: kind,
        })?;
        form.convert(value, options)
    }

    /// The BCP 47 primary language subtag (`"en"` for `"en-US"`).
    #[must_use]
    pub fn primary_subtag(&self) -> &'static str {
        self.code.split('-').next().unwrap_or(self.code)
    }

    /// Whether this is a bare-tag alias.
    #[must_use]
    pub const fn is_alias(&self) -> bool {
        matches!(self.kind, Kind::Alias { .. })
    }

    /// The full implementation this entry ultimately delegates its numerals to
    /// (itself for a full implementation).
    #[must_use]
    pub fn resolve(&self) -> &'static Language {
        match self.kind {
            Kind::Full => language(self.code).expect("every language is registered"),
            Kind::Profile { base } => language(base)
                .expect("profile base is registered")
                .resolve(),
            Kind::Alias { target } => language(target)
                .expect("alias target is registered")
                .resolve(),
        }
    }
}

/// Every entry point the crate ships, sorted by BCP 47 code.
#[must_use]
pub fn languages() -> &'static [&'static Language] {
    crate::lang::LANGUAGES
}

/// Looks a language up by BCP 47 code, ignoring case and accepting `_` for
/// `-` (`"en-US"`, `"en-us"`, `"en_US"` all resolve).
#[must_use]
pub fn language(code: &str) -> Option<&'static Language> {
    let wanted: String = code
        .trim()
        .chars()
        .map(|c| {
            if c == '_' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    languages()
        .iter()
        .copied()
        .find(|l| l.code.eq_ignore_ascii_case(&wanted))
}

/// Builds a [`Form`] for the registry from a language's typed function.
///
/// - `form!(plain to_ordinal, ORDINAL_MAX)` for a form without options;
/// - `form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)`
///   for one that takes an options struct declared with `options!`.
macro_rules! form {
    (plain $f:path, $max:expr) => {{
        fn convert(value: $crate::Number<'_>, options: &[(&str, &str)]) -> $crate::Result<String> {
            $crate::options::ensure_no_options(options)?;
            $f(value)
        }
        $crate::language::Form {
            max: $max,
            options: &[],
            convert,
        }
    }};
    (with_options $f:path, $opts:ty, $max:expr) => {{
        fn convert(value: $crate::Number<'_>, options: &[(&str, &str)]) -> $crate::Result<String> {
            let opts = <$opts>::from_pairs(options)?;
            $f(value, &opts)
        }
        $crate::language::Form {
            max: $max,
            options: <$opts>::SPEC,
            convert,
        }
    }};
}
pub(crate) use form;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_sorted_and_unique() {
        let codes: Vec<&str> = languages().iter().map(|l| l.code).collect();
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            codes, sorted,
            "registry must be sorted by code with no duplicates"
        );
        assert_eq!(codes.len(), 118);
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(language("en-US").map(|l| l.code), Some("en-US"));
        assert_eq!(language("EN-us").map(|l| l.code), Some("en-US"));
        assert_eq!(language("zh_hans_cn").map(|l| l.code), Some("zh-Hans-CN"));
        assert_eq!(language(" de ").map(|l| l.code), Some("de"));
        assert!(language("xx").is_none());
    }

    #[test]
    fn kinds_resolve_to_full_implementations() {
        for lang in languages() {
            let base = lang.resolve();
            assert_eq!(
                base.kind,
                Kind::Full,
                "{} resolves to {}, which is not a full implementation",
                lang.code,
                base.code
            );
            match lang.kind {
                Kind::Full => {}
                Kind::Profile { base } | Kind::Alias { target: base } => {
                    assert!(
                        language(base).is_some(),
                        "{} points at unknown {base}",
                        lang.code
                    );
                }
            }
        }
    }
}
