//! Currencies and the per-language currency vocabulary.
//!
//! Three independent axes meet here (see the project's `docs/language-layers.md`):
//! a language's *numerals*, the *currency words* it knows (keyed by language in
//! [`vocab`]), and each locale's *default currency*. A language can therefore
//! name any currency it has words for, not just its own default, and asking
//! for one it lacks is an [`Error::UnsupportedCurrency`], never a guess.

pub mod vocab;

use crate::error::{Error, Result};
use crate::options::{Gender, OptionKind, OptionSpec, OptionValue};
use std::fmt;
use std::str::FromStr;

macro_rules! currencies {
    ($( $(#[$meta:meta])* $code:ident = $exponent:literal ),+ $(,)?) => {
        /// An ISO 4217 currency some language in the crate can name.
        ///
        /// The variant is the ISO code. [`exponent`](Self::exponent) is the
        /// currency's minor-unit exponent: 2 for most, 0 for currencies with no
        /// everyday subunit (JPY, KRW, ...), 3 for the thousandth-unit dinars.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        #[allow(clippy::upper_case_acronyms)]
        pub enum Currency {
            $( $(#[$meta])* $code, )+
        }

        impl Currency {
            /// Every currency, in ISO code order.
            pub const ALL: &'static [Currency] = &[$(Currency::$code),+];
            /// Every ISO code, in the same order as [`ALL`](Self::ALL).
            pub const CODES: &'static [&'static str] = &[$(stringify!($code)),+];

            /// The ISO 4217 code (`"USD"`).
            #[must_use]
            pub const fn code(self) -> &'static str {
                match self { $(Currency::$code => stringify!($code),)+ }
            }

            /// The ISO 4217 minor-unit exponent: 0, 2 or 3.
            #[must_use]
            pub const fn exponent(self) -> u8 {
                match self { $(Currency::$code => $exponent,)+ }
            }
        }

        impl FromStr for Currency {
            type Err = crate::options::ParseValueError;

            /// Parses an ISO code, ignoring ASCII case (`"usd"` is `USD`).
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                $( if s.eq_ignore_ascii_case(stringify!($code)) { return Ok(Currency::$code); } )+
                Err(crate::options::ParseValueError)
            }
        }
    };
}

currencies! {
    /// UAE dirham.
    AED = 2,
    /// Australian dollar.
    AUD = 2,
    /// Azerbaijani manat.
    AZN = 2,
    /// Bangladeshi taka.
    BDT = 2,
    /// Bahraini dinar (1000 fils).
    BHD = 3,
    /// Brazilian real.
    BRL = 2,
    /// Canadian dollar.
    CAD = 2,
    /// Chinese yuan (renminbi).
    CNY = 2,
    /// Czech koruna.
    CZK = 2,
    /// Danish krone.
    DKK = 2,
    /// Algerian dinar.
    DZD = 2,
    /// Egyptian pound.
    EGP = 2,
    /// Ethiopian birr.
    ETB = 2,
    /// Euro.
    EUR = 2,
    /// Pound sterling.
    GBP = 2,
    /// Georgian lari.
    GEL = 2,
    /// Ghanaian cedi.
    GHS = 2,
    /// Hungarian forint.
    HUF = 2,
    /// Indonesian rupiah (no everyday minor unit).
    IDR = 0,
    /// Israeli new shekel.
    ILS = 2,
    /// Indian rupee.
    INR = 2,
    /// Iraqi dinar (1000 fils).
    IQD = 3,
    /// Iranian rial (no minor unit).
    IRR = 0,
    /// Jordanian dinar (1000 fils).
    JOD = 3,
    /// Japanese yen (no minor unit).
    JPY = 0,
    /// Kenyan shilling.
    KES = 2,
    /// South Korean won (no minor unit).
    KRW = 0,
    /// Kuwaiti dinar (1000 fils).
    KWD = 3,
    /// Lebanese pound.
    LBP = 2,
    /// Libyan dinar (1000 dirhams).
    LYD = 3,
    /// Moroccan dirham.
    MAD = 2,
    /// Mexican peso.
    MXN = 2,
    /// Malaysian ringgit.
    MYR = 2,
    /// Nigerian naira.
    NGN = 2,
    /// Norwegian krone.
    NOK = 2,
    /// New Zealand dollar.
    NZD = 2,
    /// Omani rial (1000 baisa).
    OMR = 3,
    /// Philippine peso.
    PHP = 2,
    /// Pakistani rupee.
    PKR = 2,
    /// Polish złoty.
    PLN = 2,
    /// Qatari riyal.
    QAR = 2,
    /// Romanian leu.
    RON = 2,
    /// Serbian dinar.
    RSD = 2,
    /// Russian ruble.
    RUB = 2,
    /// Saudi riyal.
    SAR = 2,
    /// Sudanese pound.
    SDG = 2,
    /// Swedish krona.
    SEK = 2,
    /// Singapore dollar.
    SGD = 2,
    /// Syrian pound.
    SYP = 2,
    /// Thai baht.
    THB = 2,
    /// Tunisian dinar (1000 millimes).
    TND = 3,
    /// Turkish lira.
    TRY = 2,
    /// New Taiwan dollar.
    TWD = 2,
    /// Ukrainian hryvnia.
    UAH = 2,
    /// US dollar.
    USD = 2,
    /// Vietnamese đồng (no minor unit).
    VND = 0,
    /// Yemeni rial.
    YER = 2,
    /// South African rand.
    ZAR = 2,
}

impl Currency {
    /// Decimal digits the currency parser should keep for this currency:
    /// 3 for a thousandth-unit currency, otherwise 2.
    ///
    /// This is deliberately *not* the ISO exponent: a zero-exponent currency
    /// still parses at 2 digits so that [`assert_minor_unit`] can reject the
    /// fraction instead of silently dropping it.
    #[must_use]
    pub const fn minor_unit_digits(self) -> u32 {
        if self.exponent() == 3 { 3 } else { 2 }
    }

    /// Whether the currency has an everyday minor unit.
    #[must_use]
    pub const fn has_minor_unit(self) -> bool {
        self.exponent() != 0
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl OptionValue for Currency {
    const KIND: OptionKind = OptionKind::Enum(Currency::CODES);

    fn parse(spec: &OptionSpec, raw: &str) -> Result<Self> {
        let table = match spec.kind {
            OptionKind::Currency(table) => Some(table),
            _ => None,
        };
        let Ok(currency) = raw.trim().parse::<Currency>() else {
            return Err(Error::InvalidOptionValue {
                key: spec.name,
                value: raw.to_string(),
                allowed: table.map_or_else(
                    || Currency::CODES.to_vec(),
                    |t| t.codes().map(Currency::code).collect(),
                ),
            });
        };
        if let Some(table) = table {
            table.words(currency)?;
        }
        Ok(currency)
    }

    fn render(&self) -> String {
        self.code().to_string()
    }
}

/// Refuses a nonzero minor amount in a currency that has no minor unit.
///
/// # Errors
///
/// [`Error::NoMinorUnit`] when `minor != 0` and the currency's exponent is 0.
pub fn assert_minor_unit(minor: u32, currency: Currency) -> Result<()> {
    if minor != 0 && !currency.has_minor_unit() {
        return Err(Error::NoMinorUnit { currency });
    }
    Ok(())
}

/// The word forms a language uses for one currency.
///
/// `major` and `minor` are plain lists in the language's own plural-index
/// order, not fixed singular/plural pairs: English needs two forms, Slavic
/// languages three, Arabic four, Japanese one. The language's own code
/// decides which index an amount maps to. `minor` is `None` for a currency
/// with no everyday subunit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyWords {
    /// Word forms for the major unit.
    pub major: &'static [&'static str],
    /// Word forms for the minor unit, or `None` when the currency has none.
    pub minor: Option<&'static [&'static str]>,
    /// Gender of the major-unit noun, for gender-sensitive languages.
    pub major_gender: Option<Gender>,
    /// Gender of the minor-unit noun, for gender-sensitive languages.
    pub minor_gender: Option<Gender>,
}

impl CurrencyWords {
    /// Word forms without gender information.
    #[must_use]
    pub const fn new(
        major: &'static [&'static str],
        minor: Option<&'static [&'static str]>,
    ) -> Self {
        CurrencyWords {
            major,
            minor,
            major_gender: None,
            minor_gender: None,
        }
    }

    /// Attaches the nouns' grammatical genders.
    #[must_use]
    pub const fn gendered(self, major: Gender, minor: Gender) -> Self {
        CurrencyWords {
            major_gender: Some(major),
            minor_gender: Some(minor),
            ..self
        }
    }

    /// The major form at `index`, clamped to the last form so an invariable
    /// noun (a single form) serves every index.
    #[must_use]
    pub fn major_at(&self, index: usize) -> &'static str {
        clamp(self.major, index)
    }

    /// The minor form at `index`, clamped like [`major_at`](Self::major_at).
    ///
    /// # Panics
    ///
    /// If the currency has no minor unit; call [`assert_minor_unit`] first.
    #[must_use]
    pub fn minor_at(&self, index: usize) -> &'static str {
        clamp(self.minor.expect("currency has a minor unit"), index)
    }
}

fn clamp(forms: &'static [&'static str], index: usize) -> &'static str {
    forms.get(index).or(forms.last()).copied().unwrap_or("")
}

/// The currencies one language has words for.
#[derive(Debug)]
pub struct CurrencyTable {
    /// The language key (`"en"`, `"pt-BR"`, `"zh-Hans"`).
    pub language: &'static str,
    /// The entries, in declaration order.
    pub entries: &'static [(Currency, CurrencyWords)],
}

impl CurrencyTable {
    /// A table for `language`.
    #[must_use]
    pub const fn new(
        language: &'static str,
        entries: &'static [(Currency, CurrencyWords)],
    ) -> Self {
        CurrencyTable { language, entries }
    }

    /// The word forms for `currency`, if the language has any.
    #[must_use]
    pub fn get(&self, currency: Currency) -> Option<&'static CurrencyWords> {
        self.entries
            .iter()
            .find(|(c, _)| *c == currency)
            .map(|(_, words)| words)
    }

    /// The word forms for `currency`.
    ///
    /// # Errors
    ///
    /// [`Error::UnsupportedCurrency`] listing the currencies the language knows.
    pub fn words(&self, currency: Currency) -> Result<&'static CurrencyWords> {
        self.get(currency)
            .ok_or_else(|| Error::UnsupportedCurrency {
                currency,
                allowed: self.codes().collect(),
            })
    }

    /// Whether the language has words for `currency`.
    #[must_use]
    pub fn contains(&self, currency: Currency) -> bool {
        self.get(currency).is_some()
    }

    /// The currencies the language can name, in declaration order.
    pub fn codes(&self) -> impl Iterator<Item = Currency> + '_ {
        self.entries.iter().map(|(c, _)| *c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for &c in Currency::ALL {
            assert_eq!(c.code().parse::<Currency>(), Ok(c));
            assert_eq!(c.code().to_lowercase().parse::<Currency>(), Ok(c));
            assert_eq!(c.code().len(), 3);
        }
        assert_eq!(Currency::ALL.len(), 58);
        assert!("XXX".parse::<Currency>().is_err());
        assert_eq!(Currency::TND.minor_unit_digits(), 3);
        assert_eq!(Currency::JPY.minor_unit_digits(), 2);
        assert_eq!(Currency::USD.minor_unit_digits(), 2);
        assert!(assert_minor_unit(5, Currency::JPY).is_err());
        assert!(assert_minor_unit(0, Currency::JPY).is_ok());
        assert!(assert_minor_unit(5, Currency::USD).is_ok());
    }

    #[test]
    fn tables() {
        let words = vocab::EN.words(Currency::GBP).unwrap();
        assert_eq!(words.major_at(0), "pound");
        assert_eq!(words.major_at(1), "pounds");
        assert_eq!(words.minor_at(1), "pence");
        let taka = vocab::EN.words(Currency::BDT).unwrap();
        assert_eq!(taka.major_at(1), "taka");
        assert!(matches!(
            vocab::EN.words(Currency::JPY),
            Err(Error::UnsupportedCurrency {
                currency: Currency::JPY,
                ..
            })
        ));
        assert_eq!(vocab::JA.words(Currency::JPY).unwrap().minor, None);
        assert_eq!(
            vocab::RU.words(Currency::RUB).unwrap().minor_gender,
            Some(Gender::Feminine)
        );
    }
}
