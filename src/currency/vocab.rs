//! The cross-language currency-name matrix: one table per *language* (not per
//! regional locale), keyed by the language's minimal tag.
//!
//! Word forms are plain lists in each language's own plural-index order;
//! which index an amount maps to is that language's own grammar. Portuguese
//! stays split by region ([`PT_BR`]/[`PT_PT`]) because the two spell the same
//! currency differently, and the script variants (`ZH_HANS`/`ZH_HANT`,
//! `SR_CYRL`/`SR_LATN`, `AM`/`AM_LATN`) are separate for the same reason.

use super::{Currency, CurrencyTable, CurrencyWords};
use crate::options::Gender::{Feminine, Masculine};

const fn w(major: &'static [&'static str], minor: &'static [&'static str]) -> CurrencyWords {
    CurrencyWords::new(major, Some(minor))
}

const fn no_minor(major: &'static [&'static str]) -> CurrencyWords {
    CurrencyWords::new(major, None)
}

// Word-form data reused verbatim by more than one entry within a language.
const EN_DOLLAR: CurrencyWords = w(&["dollar", "dollars"], &["cent", "cents"]);
const EN_DINAR_FILS: CurrencyWords = w(&["dinar", "dinars"], &["fils", "fils"]);
const EN_DINAR_MILLIME: CurrencyWords = w(&["dinar", "dinars"], &["millime", "millimes"]);
const EN_DIRHAM_CENTIME: CurrencyWords = w(&["dirham", "dirhams"], &["centime", "centimes"]);

/// English: every English-speaking locale's currency in one table, so any
/// English entry point can name any of them.
pub static EN: CurrencyTable = CurrencyTable::new(
    "en",
    &[
        (Currency::USD, EN_DOLLAR),
        (Currency::CAD, EN_DOLLAR),
        (Currency::AUD, EN_DOLLAR),
        (Currency::NZD, EN_DOLLAR),
        (Currency::SGD, EN_DOLLAR),
        (Currency::GBP, w(&["pound", "pounds"], &["penny", "pence"])),
        (Currency::EUR, w(&["euro"], &["cent", "cents"])), // en-IE wording
        (Currency::BDT, w(&["taka"], &["paisa", "paise"])),
        (Currency::GHS, w(&["cedi", "cedis"], &["pesewa", "pesewas"])),
        (Currency::INR, w(&["rupee", "rupees"], &["paisa", "paise"])),
        (
            Currency::KES,
            w(&["shilling", "shillings"], &["cent", "cents"]),
        ),
        (Currency::MYR, w(&["ringgit"], &["sen"])),
        (Currency::NGN, w(&["naira"], &["kobo"])),
        (
            Currency::PHP,
            w(&["peso", "pesos"], &["centavo", "centavos"]),
        ),
        (Currency::PKR, w(&["rupee", "rupees"], &["paisa", "paise"])),
        (Currency::ZAR, w(&["rand"], &["cent", "cents"])),
        // 1000-subunit dinars.
        (Currency::TND, EN_DINAR_MILLIME),
        (Currency::KWD, EN_DINAR_FILS),
        (Currency::BHD, EN_DINAR_FILS),
        (Currency::JOD, EN_DINAR_FILS),
        (Currency::IQD, EN_DINAR_FILS),
        (Currency::OMR, w(&["rial", "rials"], &["baisa", "baisas"])),
        (
            Currency::LYD,
            w(&["dinar", "dinars"], &["dirham", "dirhams"]),
        ),
        // Morocco divides into 100 centimes: an ordinary 2-decimal currency.
        (Currency::MAD, EN_DIRHAM_CENTIME),
    ],
);

/// French.
pub static FR: CurrencyTable = CurrencyTable::new(
    "fr",
    &[
        (
            Currency::EUR,
            w(&["euro", "euros"], &["centime", "centimes"]),
        ),
        (
            Currency::TND,
            w(&["dinar", "dinars"], &["millime", "millimes"]),
        ),
        (
            Currency::MAD,
            w(&["dirham", "dirhams"], &["centime", "centimes"]),
        ),
    ],
);

/// Spanish. Currency nouns are masculine.
pub static ES: CurrencyTable = CurrencyTable::new(
    "es",
    &[
        (
            Currency::EUR,
            w(&["euro", "euros"], &["céntimo", "céntimos"]).gendered(Masculine, Masculine),
        ),
        (
            Currency::MXN,
            w(&["peso", "pesos"], &["centavo", "centavos"]).gendered(Masculine, Masculine),
        ),
        (
            Currency::USD,
            w(&["dólar", "dólares"], &["centavo", "centavos"]).gendered(Masculine, Masculine),
        ),
    ],
);

/// Brazilian Portuguese.
pub static PT_BR: CurrencyTable = CurrencyTable::new(
    "pt-BR",
    &[
        (
            Currency::BRL,
            w(&["real", "reais"], &["centavo", "centavos"]),
        ),
        (
            Currency::USD,
            w(&["dólar", "dólares"], &["centavo", "centavos"]),
        ),
        (
            Currency::EUR,
            w(&["euro", "euros"], &["centavo", "centavos"]),
        ),
        (Currency::GBP, w(&["libra", "libras"], &["pêni", "pence"])),
        (Currency::JPY, no_minor(&["iene", "ienes"])),
    ],
);

/// European Portuguese.
pub static PT_PT: CurrencyTable = CurrencyTable::new(
    "pt-PT",
    &[(
        Currency::EUR,
        w(&["euro", "euros"], &["cêntimo", "cêntimos"]),
    )],
);

/// Japanese.
pub static JA: CurrencyTable = CurrencyTable::new("ja", &[(Currency::JPY, no_minor(&["円"]))]);

/// Czech: `[singular, few (2-4), many (5+)]`.
pub static CS: CurrencyTable = CurrencyTable::new(
    "cs",
    &[(
        Currency::CZK,
        w(
            &["koruna", "koruny", "korun"],
            &["haléř", "haléře", "haléřů"],
        ),
    )],
);

/// Korean.
pub static KO: CurrencyTable = CurrencyTable::new("ko", &[(Currency::KRW, no_minor(&["원"]))]);

/// Vietnamese.
pub static VI: CurrencyTable = CurrencyTable::new("vi", &[(Currency::VND, no_minor(&["đồng"]))]);

/// Persian.
pub static FA: CurrencyTable = CurrencyTable::new("fa", &[(Currency::IRR, no_minor(&["ریال"]))]);

/// Indonesian.
pub static ID: CurrencyTable = CurrencyTable::new("id", &[(Currency::IDR, no_minor(&["rupiah"]))]);

/// Bangla.
pub static BN: CurrencyTable = CurrencyTable::new("bn", &[(Currency::BDT, w(&["টাকা"], &["পয়সা"]))]);

/// Gujarati.
pub static GU: CurrencyTable = CurrencyTable::new(
    "gu",
    &[(Currency::INR, w(&["રૂપિયો", "રૂપિયા"], &["પૈસો", "પૈસા"]))],
);

// Hindi and Marathi share these Devanagari rupee/paisa forms.
const HI_MR_RUPEE: CurrencyWords = w(&["रुपया", "रुपये"], &["पैसा", "पैसे"]);

/// Hindi.
pub static HI: CurrencyTable = CurrencyTable::new("hi", &[(Currency::INR, HI_MR_RUPEE)]);

/// Kannada.
pub static KN: CurrencyTable = CurrencyTable::new(
    "kn",
    &[(Currency::INR, w(&["ರೂಪಾಯಿ", "ರೂಪಾಯಿಗಳು"], &["ಪೈಸೆ", "ಪೈಸೆಗಳು"]))],
);

/// Marathi.
pub static MR: CurrencyTable = CurrencyTable::new("mr", &[(Currency::INR, HI_MR_RUPEE)]);

/// Punjabi.
pub static PA: CurrencyTable = CurrencyTable::new(
    "pa",
    &[(Currency::INR, w(&["ਰੁਪਇਆ", "ਰੁਪਏ"], &["ਪੈਸਾ", "ਪੈਸੇ"]))],
);

/// Tamil.
pub static TA: CurrencyTable = CurrencyTable::new("ta", &[(Currency::INR, w(&["ரூபாய்"], &["பைசா"]))]);

/// Telugu.
pub static TE: CurrencyTable = CurrencyTable::new(
    "te",
    &[(Currency::INR, w(&["రూపాయి", "రూపాయలు"], &["పైసా", "పైసలు"]))],
);

/// Urdu.
pub static UR: CurrencyTable = CurrencyTable::new(
    "ur",
    &[(Currency::PKR, w(&["روپیہ", "روپے"], &["پیسہ", "پیسے"]))],
);

/// Amharic (Ge'ez script). Birr and santim are invariable.
pub static AM: CurrencyTable = CurrencyTable::new("am", &[(Currency::ETB, w(&["ብር"], &["ሳንቲም"]))]);

/// Amharic (Latin transliteration).
pub static AM_LATN: CurrencyTable =
    CurrencyTable::new("am-Latn", &[(Currency::ETB, w(&["birr"], &["santim"]))]);

/// Serbian (Cyrillic): `[singular, few (2-4), many (5+)]`. Dinar is
/// masculine, para feminine.
pub static SR_CYRL: CurrencyTable = CurrencyTable::new(
    "sr-Cyrl",
    &[(
        Currency::RSD,
        w(&["динар", "динара", "динара"], &["пара", "паре", "пара"]).gendered(Masculine, Feminine),
    )],
);

/// Serbian (Latin).
pub static SR_LATN: CurrencyTable = CurrencyTable::new(
    "sr-Latn",
    &[(
        Currency::RSD,
        w(&["dinar", "dinara", "dinara"], &["para", "pare", "para"]).gendered(Masculine, Feminine),
    )],
);

/// Simplified Chinese. `major[0]`/`[1]` are the formal/common denomination
/// words (圆/元), not singular/plural; `minor[0]`/`[1]` are jiao (角, 1/10)
/// and fen (分, 1/100), two denominations rather than plural forms.
pub static ZH_HANS: CurrencyTable = CurrencyTable::new(
    "zh-Hans",
    &[(Currency::CNY, w(&["圆", "元"], &["角", "分"]))],
);

/// Traditional Chinese, same indexing convention as [`ZH_HANS`].
pub static ZH_HANT: CurrencyTable = CurrencyTable::new(
    "zh-Hant",
    &[(Currency::TWD, w(&["圓", "元"], &["角", "分"]))],
);

// Arabic: `[singular, dual (nominative), plural (3-10), tamyīz (11-99)]`.
const AR_RIAL: &[&str] = &["ريال", "ريالان", "ريالات", "ريالاً"];
const AR_DINAR: &[&str] = &["دينار", "ديناران", "دنانير", "ديناراً"];
const AR_FILS: &[&str] = &["فلس", "فلسان", "فلوس", "فلساً"];
const AR_DIRHAM: &[&str] = &["درهم", "درهمان", "دراهم", "درهماً"];
const AR_POUND: &[&str] = &["جنيه", "جنيهان", "جنيهات", "جنيهاً"];
const AR_LIRA: &[&str] = &["ليرة", "ليرتان", "ليرات", "ليرة"];
const AR_PIASTRE: &[&str] = &["قرش", "قرشان", "قروش", "قرشاً"];
const AR_CENTIME: &[&str] = &["سنتيم", "سنتيمان", "سنتيمات", "سنتيماً"];

/// Arabic: the whole Arab-world set, since number words are shared across
/// every Arabic variant. The major units are masculine except the ليرة of
/// Lebanon and Syria.
pub static AR: CurrencyTable = CurrencyTable::new(
    "ar",
    &[
        (
            Currency::SAR,
            w(AR_RIAL, &["هللة", "هللتان", "هللات", "هللة"]).gendered(Masculine, Feminine),
        ),
        (
            Currency::TND,
            w(AR_DINAR, &["مليم", "مليمان", "مليمات", "مليماً"]).gendered(Masculine, Masculine),
        ),
        (
            Currency::KWD,
            w(AR_DINAR, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::BHD,
            w(AR_DINAR, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::JOD,
            w(AR_DINAR, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::IQD,
            w(AR_DINAR, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::OMR,
            w(AR_RIAL, &["بيسة", "بيستان", "بيسات", "بيسة"]).gendered(Masculine, Feminine),
        ),
        (
            Currency::LYD,
            w(AR_DINAR, AR_DIRHAM).gendered(Masculine, Masculine),
        ),
        // The rest divide into 100, not 1000.
        (
            Currency::MAD,
            w(AR_DIRHAM, AR_CENTIME).gendered(Masculine, Masculine),
        ),
        (
            Currency::DZD,
            w(AR_DINAR, AR_CENTIME).gendered(Masculine, Masculine),
        ),
        (
            Currency::AED,
            w(AR_DIRHAM, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::QAR,
            w(AR_RIAL, AR_DIRHAM).gendered(Masculine, Masculine),
        ),
        (
            Currency::YER,
            w(AR_RIAL, AR_FILS).gendered(Masculine, Masculine),
        ),
        (
            Currency::EGP,
            w(AR_POUND, AR_PIASTRE).gendered(Masculine, Masculine),
        ),
        (
            Currency::SDG,
            w(AR_POUND, AR_PIASTRE).gendered(Masculine, Masculine),
        ),
        (
            Currency::LBP,
            w(AR_LIRA, AR_PIASTRE).gendered(Feminine, Masculine),
        ),
        (
            Currency::SYP,
            w(AR_LIRA, AR_PIASTRE).gendered(Feminine, Masculine),
        ),
    ],
);

const SHEKEL_MAJOR: &[&str] = &["שקל", "שקלים"];

/// Biblical Hebrew: shekel and the historical gerah.
pub static HBO: CurrencyTable =
    CurrencyTable::new("hbo", &[(Currency::ILS, w(SHEKEL_MAJOR, &["גרה", "גרות"]))]);

/// Hebrew: shekel and agora.
pub static HE: CurrencyTable = CurrencyTable::new(
    "he",
    &[(Currency::ILS, w(SHEKEL_MAJOR, &["אגורה", "אגורות"]))],
);

/// Georgian. Lari and tetri are invariable.
pub static KA: CurrencyTable =
    CurrencyTable::new("ka", &[(Currency::GEL, w(&["ლარი"], &["თეთრი"]))]);

/// Croatian: `[singular, few (2-4), many (5+)]`.
pub static HR: CurrencyTable = CurrencyTable::new(
    "hr",
    &[(
        Currency::EUR,
        w(&["euro", "eura", "eura"], &["cent", "centa", "centi"]).gendered(Masculine, Masculine),
    )],
);

/// Lithuanian: `[singular, plural, genitive]`.
pub static LT: CurrencyTable = CurrencyTable::new(
    "lt",
    &[(
        Currency::EUR,
        w(&["euras", "eurai", "eurų"], &["centas", "centai", "centų"])
            .gendered(Masculine, Masculine),
    )],
);

/// Latvian: eiro is indeclinable; centi use `[singular, plural, genitive]`.
pub static LV: CurrencyTable = CurrencyTable::new(
    "lv",
    &[(
        Currency::EUR,
        w(&["eiro"], &["cents", "centi", "centu"]).gendered(Masculine, Masculine),
    )],
);

/// Polish: `[singular, few (2-4), many (5+)]`.
pub static PL: CurrencyTable = CurrencyTable::new(
    "pl",
    &[(
        Currency::PLN,
        w(
            &["złoty", "złote", "złotych"],
            &["grosz", "grosze", "groszy"],
        )
        .gendered(Masculine, Masculine),
    )],
);

/// Russian: `[singular, few, many]`. Рубль is masculine, копейка feminine.
pub static RU: CurrencyTable = CurrencyTable::new(
    "ru",
    &[(
        Currency::RUB,
        w(
            &["рубль", "рубля", "рублей"],
            &["копейка", "копейки", "копеек"],
        )
        .gendered(Masculine, Feminine),
    )],
);

/// Ukrainian: `[singular, few, many]`. Both nouns feminine.
pub static UK: CurrencyTable = CurrencyTable::new(
    "uk",
    &[(
        Currency::UAH,
        w(
            &["гривня", "гривнi", "гривень"],
            &["копiйка", "копiйки", "копiйок"],
        )
        .gendered(Feminine, Feminine),
    )],
);

/// Danish: `[singular, plural]` for krone; øre is invariable.
pub static DA: CurrencyTable =
    CurrencyTable::new("da", &[(Currency::DKK, w(&["krone", "kroner"], &["øre"]))]);

/// German: Euro and Cent are invariable in currency speech.
pub static DE: CurrencyTable =
    CurrencyTable::new("de", &[(Currency::EUR, w(&["Euro"], &["Cent"]))]);

/// Finnish: `[singular, partitive]` (euro/euroa, sentti/senttiä).
pub static FI: CurrencyTable = CurrencyTable::new(
    "fi",
    &[(Currency::EUR, w(&["euro", "euroa"], &["sentti", "senttiä"]))],
);

/// Norwegian Bokmål: `[singular, plural]` for krone; øre is invariable.
pub static NB: CurrencyTable =
    CurrencyTable::new("nb", &[(Currency::NOK, w(&["krone", "kroner"], &["øre"]))]);

/// Dutch: euro and cent are invariable.
pub static NL: CurrencyTable =
    CurrencyTable::new("nl", &[(Currency::EUR, w(&["euro"], &["cent"]))]);

/// Swedish: `[singular, plural]` for krona; öre is invariable.
pub static SV: CurrencyTable =
    CurrencyTable::new("sv", &[(Currency::SEK, w(&["krona", "kronor"], &["öre"]))]);

/// Italian: euro is invariable; `[singular, plural]` for centesimo.
pub static IT: CurrencyTable = CurrencyTable::new(
    "it",
    &[(Currency::EUR, w(&["euro"], &["centesimo", "centesimi"]))],
);

/// Romanian: `[singular, plural]`. Both nouns masculine.
pub static RO: CurrencyTable = CurrencyTable::new(
    "ro",
    &[(
        Currency::RON,
        w(&["leu", "lei"], &["ban", "bani"]).gendered(Masculine, Masculine),
    )],
);

/// Azerbaijani: manat and qəpik are invariable.
pub static AZ: CurrencyTable =
    CurrencyTable::new("az", &[(Currency::AZN, w(&["manat"], &["qəpik"]))]);

/// Greek: ευρώ is indeclinable; `[singular, plural]` for λεπτό.
pub static EL: CurrencyTable = CurrencyTable::new(
    "el",
    &[(Currency::EUR, w(&["ευρώ", "ευρώ"], &["λεπτό", "λεπτά"]))],
);

/// Filipino: piso and sentimo are invariable.
pub static FIL: CurrencyTable =
    CurrencyTable::new("fil", &[(Currency::PHP, w(&["piso"], &["sentimo"]))]);

/// Hausa: naira and kobo are invariable.
pub static HA: CurrencyTable =
    CurrencyTable::new("ha", &[(Currency::NGN, w(&["naira"], &["kobo"]))]);

/// Hungarian: forint and fillér are invariable.
pub static HU: CurrencyTable =
    CurrencyTable::new("hu", &[(Currency::HUF, w(&["forint"], &["fillér"]))]);

/// Malay: ringgit and sen are invariable.
pub static MS: CurrencyTable =
    CurrencyTable::new("ms", &[(Currency::MYR, w(&["ringgit"], &["sen"]))]);

/// Swahili: shilingi and senti are invariable.
pub static SW: CurrencyTable =
    CurrencyTable::new("sw", &[(Currency::KES, w(&["shilingi"], &["senti"]))]);

/// Thai: baht and satang are invariable.
pub static TH: CurrencyTable =
    CurrencyTable::new("th", &[(Currency::THB, w(&["บาท"], &["สตางค์"]))]);

/// Turkish: lira and kuruş are invariable.
pub static TR: CurrencyTable =
    CurrencyTable::new("tr", &[(Currency::TRY, w(&["lira"], &["kuruş"]))]);

/// Yoruba: náírà and kọ́bọ̀ are invariable.
pub static YO: CurrencyTable =
    CurrencyTable::new("yo", &[(Currency::NGN, w(&["náírà"], &["kọ́bọ̀"]))]);

/// Every table, for tooling that walks the whole matrix.
pub static ALL: &[&CurrencyTable] = &[
    &AM, &AM_LATN, &AR, &AZ, &BN, &CS, &DA, &DE, &EL, &EN, &ES, &FA, &FI, &FIL, &FR, &GU, &HA,
    &HBO, &HE, &HI, &HR, &HU, &ID, &IT, &JA, &KA, &KN, &KO, &LT, &LV, &MR, &MS, &NB, &NL, &PA, &PL,
    &PT_BR, &PT_PT, &RO, &RU, &SR_CYRL, &SR_LATN, &SV, &SW, &TA, &TE, &TH, &TR, &UK, &UR, &VI, &YO,
    &ZH_HANS, &ZH_HANT,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_is_well_formed() {
        for table in ALL {
            assert!(!table.entries.is_empty(), "{} is empty", table.language);
            for (currency, words) in table.entries {
                assert!(
                    !words.major.is_empty(),
                    "{} {currency} has no major forms",
                    table.language
                );
                assert_eq!(
                    words.minor.is_none(),
                    !currency.has_minor_unit(),
                    "{} {currency}: minor forms vs exponent",
                    table.language
                );
                let count = table.entries.iter().filter(|(c, _)| c == currency).count();
                assert_eq!(count, 1, "{} lists {currency} twice", table.language);
            }
        }
    }

    #[test]
    fn every_currency_is_named_somewhere() {
        for &c in Currency::ALL {
            assert!(ALL.iter().any(|t| t.contains(c)), "no language names {c}");
        }
    }
}
