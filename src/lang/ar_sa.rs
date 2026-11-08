//! Arabic (Saudi Arabia) (`ar-SA`): Modern Standard Arabic as used in Saudi
//! Arabia.
//!
//! A self-contained converter implementing the classical rules of the Arabic
//! counted noun (العدد والمعدود). One engine renders a bare cardinal, a scale
//! word counted by its group ("ثلاثة آلاف", "أحد عشر ألفاً", "مئتا ألف") and a
//! currency unit counted by the whole amount ("ألفا ريال وريالان"), so the
//! agreement rules live in one place:
//!
//! - 1 and 2 agree with the noun and follow it: "ريال واحد", "ريالان".
//! - 3–10 take the *opposite* gender of the noun (polarity) and govern a
//!   genitive plural: "ثلاثة ريالات" but "ثلاث هللات".
//! - 11–99 govern a singular accusative (تمييز): "أحد عشر ريالاً". The
//!   tanwīn drops when that noun is itself a construct head, so 11,000
//!   riyals is "أحد عشر ألف ريال", never "ألفاً ريال".
//! - 100, 1000 and every larger scale word govern a singular genitive
//!   (إضافة): "مئة ريال", "ألف ريال". The dual loses its ن as a construct
//!   head: "مئتا ألف", "ألفا ريال".
//! - A compound ending in 1 or 2 repeats the noun rather than fronting a
//!   bare numeral: "مئة ريال وريال واحد", as in "ألف ليلة وليلة".
//!
//! Case (الإعراب) is an option: the nominative (مرفوع) is the citation form,
//! while the accusative (منصوب) and genitive (مجرور) surface as ـين on duals
//! and tens ("اثنين وعشرين", "ألفين"). So is the spelling of 100: the
//! academy-standard مئة by default, or the traditional مائة that banknotes
//! and older texts use. Ordinals are the classical definite forms — "الحادي
//! عشر", "الحادي والعشرون", "الحادي بعد المئة" — the scheme that numbers the
//! nights of ألف ليلة وليلة.
//!
//! ```
//! use n2words::ar_sa::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(22)?, "اثنان وعشرون");
//! assert_eq!(to_ordinal(21)?, "الحادي والعشرون");
//! assert_eq!(to_currency(42.5)?, "اثنان وأربعون ريالاً وخمسون هللة");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::currency::vocab::AR as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups,
};
use crate::options::{Gender, option_enum, options};
use crate::{Error, Result};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use std::borrow::Cow;

// ============================================================================
// Vocabulary
// ============================================================================

// Units 1–19, indexed by value, in the form that agrees with a noun of the
// given gender. Polarity is baked in: the "masculine" row is what counts a
// masculine noun (ثلاثة رجال), so 3–10 carry the ة there and lack it in the
// feminine row (ثلاث نساء). 11 and 12 agree in both halves; 13–19 invert the
// unit and agree the ten (ثلاثة عشر رجلاً / ثلاث عشرة امرأة).
const ONES_MASC: [&str; 20] = [
    "",
    "واحد",
    "اثنان",
    "ثلاثة",
    "أربعة",
    "خمسة",
    "ستة",
    "سبعة",
    "ثمانية",
    "تسعة",
    "عشرة",
    "أحد عشر",
    "اثنا عشر",
    "ثلاثة عشر",
    "أربعة عشر",
    "خمسة عشر",
    "ستة عشر",
    "سبعة عشر",
    "ثمانية عشر",
    "تسعة عشر",
];
const ONES_FEM: [&str; 20] = [
    "",
    "واحدة",
    "اثنتان",
    "ثلاث",
    "أربع",
    "خمس",
    "ست",
    "سبع",
    "ثمان",
    "تسع",
    "عشر",
    "إحدى عشرة",
    "اثنتا عشرة",
    "ثلاث عشرة",
    "أربع عشرة",
    "خمس عشرة",
    "ست عشرة",
    "سبع عشرة",
    "ثماني عشرة",
    "تسع عشرة",
];

/// "One" in a compound (21, 31 …): the masculine keeps واحد, the feminine
/// takes إحدى, as in إحدى عشرة.
const fn one_in_compound(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "واحد",
        Gender::Feminine => "إحدى",
    }
}

/// The feminine 8 is a defective noun: ثمانٍ (written ثمان) in the nominative
/// and genitive, ثمانيَ in the accusative, and ثماني when it heads a plural
/// (ثماني هللات).
const EIGHT_FEM_CONSTRUCT: &str = "ثماني";

/// The "ten" of 11–19 agrees with the noun in both cardinals (اثنا عشر /
/// اثنتا عشرة) and ordinals (الحادي عشر / الحادية عشرة).
const fn teen_ten(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "عشر",
        Gender::Feminine => "عشرة",
    }
}

// Tens 20–90 without their case ending: ـون nominative, ـين oblique.
const TENS_STEM: [&str; 10] = [
    "", "", "عشر", "ثلاث", "أربع", "خمس", "ست", "سبع", "ثمان", "تسع",
];

/// The case ending of the tens: ـون in the nominative, ـين otherwise.
const fn tens_ending(case: Case) -> &'static str {
    match case {
        Case::Nominative => "ون",
        Case::Accusative | Case::Genitive => "ين",
    }
}

// 300–900 are one word: the unit fused to the hundred (ثلاثمئة / ثلاثمائة,
// whichever spelling is in force). 200 is the dual of the hundred (مئتان /
// مائتان; مئتين oblique; مئتا / مئتي as a construct head), derived below.
const HUNDREDS_UNIT: [&str; 10] = ["", "", "", "ثلاث", "أربع", "خمس", "ست", "سبع", "ثمان", "تسع"];

// Scale words above the units group, index 1 = 10^3. Every one is a
// masculine noun; the singular and the plural counted by 3–10 are listed,
// the dual and tamyīz forms are derived from the singular.
const SCALES: &[&str] = &[
    "",
    "ألف",
    "مليون",
    "مليار",
    "تريليون",
    "كوادريليون",
    "كوينتليون",
    "سكستيليون",
];
const SCALE_PLURALS: &[&str] = &[
    "",
    "آلاف",
    "ملايين",
    "مليارات",
    "تريليونات",
    "كوادريليونات",
    "كوينتليونات",
    "سكستيليونات",
];

/// The cardinal ceiling, derived from the scale table: `10^24`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The ordinal ceiling: `10^24`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The currency ceiling: `10^24`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 - 1);

const ZERO: &str = "صفر";
const NEGATIVE: &str = "ناقص";
const DECIMAL_SEP: &str = "فاصلة";
const AND: &str = "و";
const AFTER: &str = "بعد";
const DEFINITE: &str = "ال";

// Ordinals 1–10 (index = value). "First" has a compound form, الحادي, used in
// 11, 21 … and after مئة/ألف; الأول is only ever the standalone word.
const ORDINAL_MASC: [&str; 11] = [
    "",
    "الأول",
    "الثاني",
    "الثالث",
    "الرابع",
    "الخامس",
    "السادس",
    "السابع",
    "الثامن",
    "التاسع",
    "العاشر",
];
const ORDINAL_FEM: [&str; 11] = [
    "",
    "الأولى",
    "الثانية",
    "الثالثة",
    "الرابعة",
    "الخامسة",
    "السادسة",
    "السابعة",
    "الثامنة",
    "التاسعة",
    "العاشرة",
];

/// "First" in compound position (الحادي عشر, الحادي والعشرون).
const fn ordinal_first_compound(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "الحادي",
        Gender::Feminine => "الحادية",
    }
}

option_enum! {
    /// Grammatical case (الإعراب) of the number words.
    pub enum Case {
        /// Nominative (مرفوع), the citation form: اثنان وعشرون.
        Nominative = "nominative",
        /// Accusative (منصوب): ـين on duals and tens, اثنين وعشرين.
        Accusative = "accusative",
        /// Genitive (مجرور): ـين on duals and tens, اثنين وعشرين.
        Genitive = "genitive",
    }
}

option_enum! {
    /// Spelling of 100, which 200–900 inherit.
    pub enum HundredSpelling {
        /// The academy-standard مئة.
        Mia = "مئة",
        /// The traditional مائة, as on banknotes and in older texts.
        Miaa = "مائة",
    }
}

/// The option-driven choices every number word is rendered under, bundled so
/// the builders take one argument instead of threading each separately.
#[derive(Debug, Clone, Copy)]
struct Style {
    /// Case of the number.
    case: Case,
    /// Spelling of 100, which 200–900 inherit.
    hundred: HundredSpelling,
}

// ============================================================================
// Counted nouns
// ============================================================================

/// A noun as the number engine sees it: the four forms Arabic counting
/// selects between, plus the gender that drives agreement of 1–19.
struct Noun {
    /// Bare singular: after 100/1000… and in "X واحد".
    singular: &'static str,
    /// Nominative dual with its ن: "ريالان", "ألفان".
    dual: Cow<'static, str>,
    /// Plural counted by 3–10: "ريالات", "آلاف".
    plural: &'static str,
    /// Singular accusative after 11–99: "ريالاً".
    tamyiz: Cow<'static, str>,
    /// Grammatical gender of the noun.
    gender: Gender,
    /// Whether one is spelled out ("ريال واحد") or the bare noun stands
    /// alone ("ألف").
    counts_with_one: bool,
}

/// The counted-noun view of a scale word: every scale word is masculine,
/// forms its dual and tamyīz regularly, and stands alone for one ("ألف",
/// never "ألف واحد"). `level` indexes [`SCALES`] (1 = thousand).
fn scale_noun(level: usize) -> Noun {
    let singular = SCALES[level];
    Noun {
        singular,
        dual: Cow::Owned(format!("{singular}ان")),
        plural: SCALE_PLURALS[level],
        tamyiz: Cow::Owned(format!("{singular}اً")),
        gender: Gender::Masculine,
        counts_with_one: false,
    }
}

/// The counted-noun view of a currency unit, from its currency-vocab row
/// (`[singular, dual, plural (3–10), tamyīz (11+)]`).
fn currency_noun(forms: &'static [&'static str], gender: Gender) -> Noun {
    // Clamp to the last form so a shorter row still yields words, never a panic.
    let at = |index: usize| forms.get(index).or(forms.last()).copied().unwrap_or("");
    Noun {
        singular: at(0),
        dual: Cow::Borrowed(at(1)),
        plural: at(2),
        tamyiz: Cow::Borrowed(at(3)),
        gender,
        counts_with_one: true,
    }
}

/// `s` without its last `count` characters (the JS `slice(0, -count)`).
fn drop_last(s: &str, count: usize) -> &str {
    let keep = s.chars().count().saturating_sub(count);
    let end = s.char_indices().nth(keep).map_or(s.len(), |(i, _)| i);
    &s[..end]
}

/// Inflect a sound dual for case: the nominative ـان becomes ـين in the
/// accusative and genitive ("ريالان" → "ريالين").
fn inflect_dual(dual: &str, case: Case) -> String {
    if case == Case::Nominative {
        dual.to_string()
    } else {
        format!("{}ين", drop_last(dual, 2))
    }
}

/// The dual as a construct head (مضاف): the ن drops, leaving ـا in the
/// nominative and ـي otherwise ("ألفا ريال", "ألفي ريال").
fn construct_dual(dual: &str, case: Case) -> String {
    let ending = if case == Case::Nominative { "ا" } else { "ي" };
    format!("{}{ending}", drop_last(dual, 2))
}

/// The noun counted by one: "ريال واحد" for a currency, the bare "ألف" for a
/// scale word.
fn noun_for_one(noun: &Noun) -> String {
    if !noun.counts_with_one {
        return noun.singular.to_string();
    }
    let one = match noun.gender {
        Gender::Feminine => ONES_FEM[1],
        Gender::Masculine => ONES_MASC[1],
    };
    format!("{} {one}", noun.singular)
}

// ============================================================================
// Number words
// ============================================================================

/// Render 1–99 as conjuncts (joined later with و): ["خمسة"], ["خمسة", "عشرون"],
/// ["أحد عشر"]. The gender is the counted noun's; the tables carry polarity.
/// `construct` says whether 3–10 heads a genitive plural (ثماني هللات).
fn tens_and_ones(r: u32, gender: Gender, style: Style, construct: bool) -> Vec<String> {
    let case = style.case;
    let feminine = gender == Gender::Feminine;
    let ones: &[&str; 20] = if feminine { &ONES_FEM } else { &ONES_MASC };
    let unit = (r % 10) as usize;

    if r < 20 {
        // The only declinable parts of 1–19 are the duals (2, and the ن-less
        // first half of 12: اثنا عشر / اثني عشر) and the defective feminine 8;
        // 11 and 13–19 are indeclinable compounds.
        if r == 2 {
            return vec![inflect_dual(ones[2], case)];
        }
        if r == 12 {
            return vec![format!(
                "{} {}",
                construct_dual(ones[2], case),
                teen_ten(gender)
            )];
        }
        if r == 8 && feminine && (construct || case == Case::Accusative) {
            return vec![EIGHT_FEM_CONSTRUCT.to_string()];
        }
        return vec![ones[r as usize].to_string()];
    }

    let tens_word = format!("{}{}", TENS_STEM[(r / 10) as usize], tens_ending(case));
    if unit == 0 {
        return vec![tens_word];
    }

    let unit_word = if unit == 1 {
        one_in_compound(gender).to_string()
    } else if unit == 2 {
        inflect_dual(ones[2], case)
    } else if unit == 8 && feminine && case == Case::Accusative {
        EIGHT_FEM_CONSTRUCT.to_string()
    } else {
        ones[unit].to_string()
    };
    vec![unit_word, tens_word]
}

/// The hundreds word of a group: the hundred itself, its dual (or the dual's
/// construct form when it heads the following noun: "مئتا ألف"), or one of
/// the fused 300–900 — all in the spelling in force. `h` is the hundreds
/// digit, 1–9; `construct` says whether the word directly heads its noun.
fn hundreds_word(h: u32, style: Style, construct: bool) -> String {
    let hundred = style.hundred.as_str();
    if h == 1 {
        return hundred.to_string();
    }
    if h != 2 {
        return format!("{}{hundred}", HUNDREDS_UNIT[h as usize]);
    }
    // The dual replaces the ة: مئة → مئتان, مائة → مائتان.
    let dual = format!("{}تان", drop_last(hundred, 1));
    if construct {
        construct_dual(&dual, style.case)
    } else {
        inflect_dual(&dual, style.case)
    }
}

/// Render a bare 1–999 with no noun attached (the units group of a plain
/// cardinal).
fn group_words(g: u32, gender: Gender, style: Style) -> Vec<String> {
    let h = g / 100;
    let r = g % 100;
    let mut parts = if h > 0 {
        vec![hundreds_word(h, style, false)]
    } else {
        Vec::new()
    };
    if r > 0 {
        parts.extend(tens_and_ones(r, gender, style, false));
    }
    parts
}

/// Render 1–999 counting a noun, the noun in the form its last number word
/// governs. When `construct` is set the noun is itself the head of a following
/// genitive (a scale word before a currency unit), so it takes its construct
/// forms: the dual drops its ن and the tamyīz loses its tanwīn.
fn group_with_noun(g: u32, noun: &Noun, style: Style, construct: bool) -> Vec<String> {
    let h = g / 100;
    let r = g % 100;
    let dual = if construct {
        construct_dual(&noun.dual, style.case)
    } else {
        inflect_dual(&noun.dual, style.case)
    };

    if g == 1 {
        return vec![noun_for_one(noun)];
    }
    if g == 2 {
        return vec![dual];
    }

    // Exact hundreds, and hundreds followed by 1 or 2: the hundreds word heads
    // the noun in the singular ("مئة ريال"), and a trailing one or two repeats
    // the noun rather than fronting a bare numeral ("مئة ريال وريال واحد").
    let hundreds_of_noun = if h > 0 {
        format!("{} {}", hundreds_word(h, style, true), noun.singular)
    } else {
        String::new()
    };
    if r == 0 {
        return vec![hundreds_of_noun];
    }
    if h > 0 && r == 1 {
        return vec![hundreds_of_noun, noun_for_one(noun)];
    }
    if h > 0 && r == 2 {
        return vec![hundreds_of_noun, dual];
    }

    // 3–10 take the plural; 11–99 the singular accusative (tamyīz), whose
    // tanwīn a construct head sheds ("أحد عشر ألفاً" but "أحد عشر ألف ريال").
    let noun_form: &str = if r <= 10 {
        noun.plural
    } else if construct {
        noun.singular
    } else {
        &noun.tamyiz
    };
    let mut parts = if h > 0 {
        vec![hundreds_word(h, style, false)]
    } else {
        Vec::new()
    };
    parts.extend(tens_and_ones(r, noun.gender, style, r <= 10));
    if let Some(last) = parts.last_mut() {
        last.push(' ');
        last.push_str(noun_form);
    }
    parts
}

/// Render a positive integer as conjuncts, optionally counting a noun that
/// follows the whole number (the currency unit). Higher groups count their
/// scale word; the noun attaches to the lowest group, or to the group above
/// it when the units are 1 or 2 ("ألف ريال وريال واحد"). `gender` is what the
/// units agree with when there is no noun.
fn integer_to_conjuncts(
    n: &BigUint,
    gender: Gender,
    style: Style,
    noun: Option<&Noun>,
) -> Vec<String> {
    let groups = split_groups(n, 1000);

    // Which group's scale word directly heads the noun: normally the lowest
    // non-zero group, but units of 1 or 2 defer to the group above and are
    // appended as "وريال واحد" / "وريالان" after it.
    let units_repeat_noun = noun.is_some() && groups.len() > 1 && groups[0] > 0 && groups[0] <= 2;
    let head_level = if noun.is_none() {
        None
    } else {
        groups
            .iter()
            .enumerate()
            .position(|(level, &g)| g > 0 && (level > 0 || !units_repeat_noun))
    };

    let mut parts = Vec::new();
    for level in (0..groups.len()).rev() {
        let g = groups[level];
        if g == 0 {
            continue;
        }
        if level == 0 {
            match noun {
                None => parts.extend(group_words(g, gender, style)),
                Some(noun) if units_repeat_noun => parts.push(if g == 1 {
                    noun_for_one(noun)
                } else {
                    inflect_dual(&noun.dual, style.case)
                }),
                Some(noun) => parts.extend(group_with_noun(g, noun, style, false)),
            }
            continue;
        }
        let heads = head_level == Some(level);
        parts.extend(group_with_noun(g, &scale_noun(level), style, heads));
        if let (true, Some(noun), Some(last)) = (heads, noun, parts.last_mut()) {
            last.push(' ');
            last.push_str(noun.singular);
        }
    }
    parts
}

/// Join conjuncts with و, which attaches to the following word: "مئة وخمسة".
fn join_conjuncts(parts: &[String]) -> String {
    parts.join(&format!(" {AND}"))
}

/// Convert a non-negative integer to Arabic words.
fn integer_to_words(n: &BigUint, gender: Gender, style: Style) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    join_conjuncts(&integer_to_conjuncts(n, gender, style, None))
}

/// Convert the fractional digits of a number to Arabic words: leading zeros
/// digit by digit, then the remainder as one integer.
fn decimal_part_to_words(decimal: &str, gender: Gender, style: Style) -> String {
    let mut parts = Vec::new();
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        parts.push(ZERO.to_string());
    }
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        parts.push(integer_to_words(
            &digits_to_biguint(remainder),
            gender,
            style,
        ));
    }
    parts.join(" ")
}

// ============================================================================
// Cardinal
// ============================================================================

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Grammatical gender of the counted noun
        gender: Gender = Gender::Masculine;
        /// Grammatical case (الإعراب): nominative/مرفوع (اثنان وعشرون), accusative/منصوب or genitive/مجرور (اثنين وعشرين)
        case: Case = Case::Nominative;
        /// Spelling of 100: the academy-standard مئة or the traditional مائة, as on banknotes
        hundred_spelling: HundredSpelling = HundredSpelling::Mia;
        /// Custom word for negative numbers
        negative_word: String = NEGATIVE.to_string();
    }
}

/// Spells a number in Arabic words.
///
/// ```
/// # use n2words::ar_sa::to_cardinal;
/// assert_eq!(to_cardinal(1)?, "واحد");
/// assert_eq!(to_cardinal(300)?, "ثلاثمئة");
/// assert_eq!(to_cardinal(-20)?, "ناقص عشرون");
/// assert_eq!(to_cardinal(17.42)?, "سبعة عشر فاصلة اثنان وأربعون");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CARDINAL_MAX`].
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_cardinal_with(value, &CardinalOptions::default())
}

/// Spells a number in Arabic words, with options.
///
/// ```
/// # use n2words::ar_sa::{to_cardinal_with, CardinalOptions, Case, HundredSpelling};
/// # use n2words::Gender;
/// let feminine = CardinalOptions { gender: Gender::Feminine, ..Default::default() };
/// assert_eq!(to_cardinal_with(1, &feminine)?, "واحدة");
/// let genitive = CardinalOptions { case: Case::Genitive, ..Default::default() };
/// assert_eq!(to_cardinal_with(22, &genitive)?, "اثنين وعشرين");
/// let banknote = CardinalOptions { hundred_spelling: HundredSpelling::Miaa, ..Default::default() };
/// assert_eq!(to_cardinal_with(300, &banknote)?, "ثلاثمائة");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_cardinal`].
pub fn to_cardinal_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CardinalOptions,
) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;
    let style = Style {
        case: options.case,
        hundred: options.hundred_spelling,
    };

    let mut parts = Vec::new();
    if number.negative {
        parts.push(options.negative_word.clone());
    }
    parts.push(integer_to_words(&number.integer, options.gender, style));
    if let Some(fraction) = &number.fraction {
        parts.push(DECIMAL_SEP.to_string());
        parts.push(decimal_part_to_words(fraction, options.gender, style));
    }
    Ok(parts.join(" "))
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal for 1–99: the dedicated forms for 1–10, compounds for 11–19
/// ("الحادي عشر"), and unit + definite ten for 21–99 ("الحادي والعشرون").
/// `compound` says whether "first" is in compound position (الحادي, not الأول).
fn small_ordinal(r: u32, gender: Gender, case: Case, compound: bool) -> String {
    let ordinals: &[&str; 11] = if gender == Gender::Feminine {
        &ORDINAL_FEM
    } else {
        &ORDINAL_MASC
    };
    let unit = (r % 10) as usize;
    let unit_word = if unit == 1 {
        ordinal_first_compound(gender)
    } else {
        ordinals[unit]
    };

    if r <= 10 {
        return if compound && r == 1 {
            unit_word
        } else {
            ordinals[r as usize]
        }
        .to_string();
    }
    if r < 20 {
        return format!("{unit_word} {}", teen_ten(gender));
    }

    let tens_word = format!(
        "{DEFINITE}{}{}",
        TENS_STEM[(r / 10) as usize],
        tens_ending(case)
    );
    if unit == 0 {
        tens_word
    } else {
        format!("{unit_word} {AND}{tens_word}")
    }
}

/// The definite cardinal ("المئة", "الألف والمئتين", "الثلاثة آلاف"): the
/// article goes on the first word of every conjunct.
fn definite_cardinal(n: &BigUint, style: Style) -> String {
    let parts: Vec<String> = integer_to_conjuncts(n, Gender::Masculine, style, None)
        .into_iter()
        .map(|part| format!("{DEFINITE}{part}"))
        .collect();
    join_conjuncts(&parts)
}

/// Gets the Arabic ordinal for a positive integer. Past 99 Arabic has no
/// dedicated forms: an exact multiple of 100 is the definite cardinal
/// ("المئة", "الألفان"), and anything else is the ordinal of the last two
/// digits followed by بعد and the genitive definite remainder — "الحادي
/// والعشرون بعد المئة", as the nights of ألف ليلة وليلة are numbered.
fn integer_to_ordinal(n: &BigUint, gender: Gender, style: Style) -> String {
    let r = (n % 100u32).to_u32().expect("remainder below 100");
    let base = n - BigUint::from(r);
    if base.is_zero() {
        return small_ordinal(r, gender, style.case, false);
    }
    if r == 0 {
        return definite_cardinal(&base, style);
    }
    let genitive = Style {
        case: Case::Genitive,
        ..style
    };
    format!(
        "{} {AFTER} {}",
        small_ordinal(r, gender, style.case, true),
        definite_cardinal(&base, genitive)
    )
}

options! {
    /// Options for [`to_ordinal_with`].
    pub struct OrdinalOptions {
        /// Grammatical gender of the ordinal
        gender: Gender = Gender::Masculine;
        /// Grammatical case (الإعراب): nominative/مرفوع (الحادي والعشرون), accusative/منصوب or genitive/مجرور (الحادي والعشرين)
        case: Case = Case::Nominative;
        /// Spelling of 100: the academy-standard مئة or the traditional مائة, as on banknotes
        hundred_spelling: HundredSpelling = HundredSpelling::Mia;
    }
}

/// Spells a positive integer as an Arabic ordinal.
///
/// ```
/// # use n2words::ar_sa::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "الأول");
/// assert_eq!(to_ordinal(21)?, "الحادي والعشرون");
/// assert_eq!(to_ordinal(101)?, "الحادي بعد المئة");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction; [`Error::OutOfRange`](crate::Error::OutOfRange) at
/// or past [`ORDINAL_MAX`].
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_ordinal_with(value, &OrdinalOptions::default())
}

/// Spells a positive integer as an Arabic ordinal, with options.
///
/// ```
/// # use n2words::ar_sa::{to_ordinal_with, Case, OrdinalOptions};
/// # use n2words::Gender;
/// let feminine = OrdinalOptions { gender: Gender::Feminine, ..Default::default() };
/// assert_eq!(to_ordinal_with(1, &feminine)?, "الأولى");
/// let genitive = OrdinalOptions { case: Case::Genitive, ..Default::default() };
/// assert_eq!(to_ordinal_with(121, &genitive)?, "الحادي والعشرين بعد المئة");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_ordinal`].
pub fn to_ordinal_with<'a>(
    value: impl Into<Number<'a>>,
    options: &OrdinalOptions,
) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    ORDINAL_MAX.check(&n, None)?;
    let style = Style {
        case: options.case,
        hundred: options.hundred_spelling,
    };
    Ok(integer_to_ordinal(&n, options.gender, style))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::SAR, table = CURRENCY_VOCAB;
        /// Grammatical case (الإعراب): nominative/مرفوع (ريالان) or, as after مبلغ, accusative/منصوب or genitive/مجرور (ريالين)
        case: Case = Case::Nominative;
        /// Spelling of 100: the academy-standard مئة or the traditional مائة, as on banknotes
        hundred_spelling: HundredSpelling = HundredSpelling::Mia;
    }
}

/// Spells an amount in Saudi riyals.
///
/// ```
/// # use n2words::ar_sa::to_currency;
/// assert_eq!(to_currency(1)?, "ريال واحد");
/// assert_eq!(to_currency(2000)?, "ألفا ريال");
/// assert_eq!(to_currency(0.01)?, "هللة واحدة");
/// assert_eq!(to_currency(1001.01)?, "ألف ريال وريال واحد وهللة واحدة");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CURRENCY_MAX`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Arabic has words for, with options.
///
/// ```
/// # use n2words::{ar_sa::{to_currency_with, Case, CurrencyOptions}, Currency};
/// let tunisian = CurrencyOptions { currency: Currency::TND, ..Default::default() };
/// assert_eq!(to_currency_with("1.500", &tunisian)?, "دينار واحد وخمسمئة مليم");
/// let genitive = CurrencyOptions { case: Case::Genitive, ..Default::default() };
/// assert_eq!(to_currency_with(2.02, &genitive)?, "ريالين وهللتين");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Arabic has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one.
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    // The currency decides how many decimal digits the parser keeps: a
    // 1000-subunit currency (TND, KWD, ...) read at 2 would turn "1.500" into
    // 50 minor units instead of 500.
    let words = CURRENCY_VOCAB.words(options.currency)?;
    let money = parse_currency(&value.into(), options.currency.minor_unit_digits())?;
    CURRENCY_MAX.check(&money.major, None)?; // minor units are <= 999, safe
    assert_minor_unit(money.minor, options.currency)?;
    let style = Style {
        case: options.case,
        hundred: options.hundred_spelling,
    };

    let mut parts = Vec::new();
    if !money.major.is_zero() || money.minor == 0 {
        // Every Arabic entry carries both genders; masculine is the fallback
        // a table row without one would get.
        let noun = currency_noun(words.major, words.major_gender.unwrap_or(Gender::Masculine));
        if money.major.is_zero() {
            parts.push(format!("{ZERO} {}", noun.singular));
        } else {
            parts.extend(integer_to_conjuncts(
                &money.major,
                noun.gender,
                style,
                Some(&noun),
            ));
        }
    }
    if money.minor > 0 {
        // `assert_minor_unit` has refused a fraction in a currency without a
        // minor unit, so the row has forms here; a row without them is an
        // error, never a panic.
        let forms = words.minor.ok_or(Error::NoMinorUnit {
            currency: options.currency,
        })?;
        let noun = currency_noun(forms, words.minor_gender.unwrap_or(Gender::Masculine));
        parts.extend(integer_to_conjuncts(
            &BigUint::from(money.minor),
            noun.gender,
            style,
            Some(&noun),
        ));
    }

    let words = join_conjuncts(&parts);
    Ok(if money.negative {
        format!("{NEGATIVE} {words}")
    } else {
        words
    })
}

/// Registry entry for `ar-SA`.
pub static LANGUAGE: Language = Language {
    code: "ar-SA",
    name: "Arabic (Saudi Arabia)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
