//! Range ceilings: how large a value each form can spell.
//!
//! Every form declares the smallest value it refuses as a [`Max`], derived
//! from its own scale table through one of the constructors below so the
//! ceiling tracks the vocabulary and cannot drift. Past the largest scale
//! word a form returns [`Error::OutOfRange`] rather than inventing words.

use crate::error::{Error, Result};
use num_bigint::BigUint;
use num_traits::{One, Pow, Zero};
use std::fmt;

/// A form's ceiling: the smallest value it refuses to convert.
///
/// Every ceiling in the crate is an exact power of ten (so the largest
/// convertible value is `10^n - 1`), or there is none at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Max {
    /// Values from `10^n` upwards are refused.
    Pow10(u32),
    /// No fixed ceiling: recursive or compounding spellers (Persian, Thai, ...).
    Unbounded,
}

impl Max {
    /// 3-digit grouping with a scale table starting at "thousand" (English,
    /// German, Russian, ...): `scale_count` scale words give `10^((n + 1) * 3)`.
    #[must_use]
    pub const fn western(scale_count: u32) -> Max {
        Max::Pow10((scale_count + 1) * 3)
    }

    /// Myriad (4-digit) grouping, one scale word per power of 10,000
    /// (Japanese, Korean): `10^((n + 1) * 4)`.
    #[must_use]
    pub const fn myriad(scale_count: u32) -> Max {
        Max::Pow10((scale_count + 1) * 4)
    }

    /// South Asian 3-2-2 grouping: a 3-digit base segment, then two digits per
    /// scale word, where the table's index 0 is the (empty) units slot.
    /// `word_count` is the table's length including that slot.
    #[must_use]
    pub const fn indian(word_count: u32) -> Max {
        Max::Pow10(3 + 2 * (word_count - 1))
    }

    /// Long scale: each base word yields an -illion and an -illiard, spanning
    /// six powers of ten (Spanish, French, Italian): `10^(6 * (n + 1))`.
    #[must_use]
    pub const fn long_scale(unit_count: u32) -> Max {
        Max::Pow10(6 * (unit_count + 1))
    }

    /// A structural bound given directly as its base-10 exponent.
    #[must_use]
    pub const fn bounded(exponent: u32) -> Max {
        Max::Pow10(exponent)
    }

    /// The base-10 exponent of the ceiling, or `None` when unbounded.
    #[must_use]
    pub const fn exponent(self) -> Option<u32> {
        match self {
            Max::Pow10(e) => Some(e),
            Max::Unbounded => None,
        }
    }

    /// Whether the form has a finite ceiling.
    #[must_use]
    pub const fn is_bounded(self) -> bool {
        matches!(self, Max::Pow10(_))
    }

    /// The ceiling itself (`10^n`), or `None` when unbounded.
    #[must_use]
    pub fn value(self) -> Option<BigUint> {
        self.exponent().map(pow10)
    }

    /// The largest convertible value (`10^n - 1`), or `None` when unbounded.
    #[must_use]
    pub fn largest(self) -> Option<BigUint> {
        self.value().map(|v| v - BigUint::one())
    }

    /// Renders the largest value as `10^N - 1`, or `∞` when unbounded.
    #[must_use]
    pub fn largest_display(self) -> String {
        match self {
            Max::Pow10(e) => format!("10^{e} - 1"),
            Max::Unbounded => "∞".to_string(),
        }
    }

    /// Refuses `value`, or a fraction spelled through the same scale builder,
    /// when it reaches the ceiling.
    ///
    /// Pass `fraction` (the decimal digit string) **only** when the fraction is
    /// spelled as a whole number via the scale builder; digit-by-digit
    /// languages and the ordinal/currency forms omit it so long fractions stay
    /// valid. The in-range path is a cheap bit-length comparison.
    ///
    /// # Errors
    ///
    /// [`Error::OutOfRange`] when `value >= 10^n`, or the fraction read as an
    /// integer is.
    pub fn check(self, value: &BigUint, fraction: Option<&str>) -> Result<()> {
        let Max::Pow10(exponent) = self else {
            return Ok(());
        };
        if reaches(value, exponent) {
            return Err(Error::OutOfRange { max: self });
        }
        if let Some(digits) = fraction {
            // The fraction's significant digit count is its value's digit count.
            let significant = digits.trim_start_matches('0').len();
            if significant > exponent as usize {
                return Err(Error::OutOfRange { max: self });
            }
        }
        Ok(())
    }
}

impl fmt::Display for Max {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Max::Pow10(e) => write!(f, "10^{e}"),
            Max::Unbounded => f.write_str("∞"),
        }
    }
}

/// `10^exponent` as a big integer.
#[must_use]
pub fn pow10(exponent: u32) -> BigUint {
    BigUint::from(10u32).pow(exponent)
}

/// `value >= 10^exponent`, deciding by bit length where possible.
fn reaches(value: &BigUint, exponent: u32) -> bool {
    if value.is_zero() {
        return false;
    }
    // log2(10) bounds, scaled by 1000 to stay in integer arithmetic.
    const LOG2_10_LOW: u64 = 3321; // < log2(10) * 1000
    const LOG2_10_HIGH: u64 = 3322; // > log2(10) * 1000
    let bits = value.bits();
    let exponent = u64::from(exponent);
    // value < 2^bits <= 10^exponent whenever bits * 1000 <= exponent * 3321.
    if bits * 1000 <= exponent * LOG2_10_LOW {
        return false;
    }
    // value >= 2^(bits-1) >= 10^exponent whenever (bits-1) * 1000 >= exponent * 3322.
    if (bits - 1) * 1000 >= exponent * LOG2_10_HIGH {
        return true;
    }
    *value >= pow10(exponent as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_match_the_javascript_helpers() {
        assert_eq!(Max::western(21), Max::Pow10(66));
        assert_eq!(Max::myriad(17), Max::Pow10(72));
        assert_eq!(Max::indian(9), Max::Pow10(19));
        assert_eq!(Max::long_scale(4), Max::Pow10(30));
        assert_eq!(Max::bounded(16), Max::Pow10(16));
    }

    #[test]
    fn check_pins_the_exact_boundary() {
        for e in 1..80u32 {
            let max = Max::Pow10(e);
            let ceiling = pow10(e);
            assert!(
                max.check(&(&ceiling - BigUint::one()), None).is_ok(),
                "10^{e} - 1 must pass"
            );
            assert_eq!(
                max.check(&ceiling, None),
                Err(Error::OutOfRange { max }),
                "10^{e} must fail"
            );
            assert!(max.check(&(&ceiling + BigUint::one()), None).is_err());
        }
        assert!(Max::Pow10(3).check(&BigUint::zero(), None).is_ok());
        assert!(Max::Unbounded.check(&pow10(500), Some("1234567")).is_ok());
    }

    #[test]
    fn check_applies_to_integer_spelled_fractions() {
        let max = Max::Pow10(3);
        assert!(max.check(&BigUint::from(5u32), Some("999")).is_ok());
        assert!(max.check(&BigUint::from(5u32), Some("000999")).is_ok());
        assert!(max.check(&BigUint::from(5u32), Some("1000")).is_err());
    }

    #[test]
    fn display() {
        assert_eq!(Max::Pow10(66).largest_display(), "10^66 - 1");
        assert_eq!(Max::Unbounded.largest_display(), "∞");
        assert_eq!(Max::Pow10(9).largest(), Some(BigUint::from(999_999_999u32)));
        assert_eq!(Max::Unbounded.value(), None);
    }
}
