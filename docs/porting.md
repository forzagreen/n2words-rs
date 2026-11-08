# Porting a language from the JavaScript n2words

This crate is a port of [forzagreen/n2words](https://github.com/forzagreen/n2words)
(JavaScript). Each JS file `src/{code}.js` becomes one Rust module
`src/lang/{module}.rs`, where `module` is the BCP 47 code lowercased with `-`
replaced by `_` (`en-US` → `en_us`, `zh-Hans-CN` → `zh_hans_cn`). The module
list and registry in `src/lang/mod.rs` are generated and already list every
language; a language that is not ported yet is a **stub** (`LANGUAGE` with every
form `None`). Porting means replacing the stub's contents.

The test fixtures are already imported (`tests/fixtures/{code}.json`, from the
JS `test/fixtures/{code}.js`), so a port is done when its fixtures pass and the
generic gates in `tests/contracts.rs` pass.

## Reference ports

| JS shape | Rust reference | Notes |
| --- | --- | --- |
| Full implementation, cardinal **with** options | `src/lang/en_us.rs` | `to_cardinal` + `to_cardinal_with`, `options!` struct |
| Full implementation, cardinal **without** options | `src/lang/en_gb.rs` | `to_cardinal` only |
| Locale profile (`variantOf`) | `src/lang/en_au.rs` | re-exports base numerals, own `CurrencyOptions` default |
| Bare-tag alias (`aliasOf`) | `src/lang/en.rs` | re-exports target, `required currency` |

Read the reference matching your file's shape before starting. Keep the same
section order and the same level of doc comments.

## The three kinds of module

**Full implementation** (`Kind::Full`): owns its vocabulary and builders.
Exports, for each form it implements:

- `pub const CARDINAL_MAX: Max` / `ORDINAL_MAX` / `CURRENCY_MAX`, derived from the
  scale table with `Max::western(n)`, `Max::myriad(n)`, `Max::indian(n)`,
  `Max::long_scale(n)`, `Max::bounded(e)` or `Max::Unbounded` — exactly the JS
  `western(...)` / `myriad(...)` / `indian(...)` / `longScale(...)` /
  `bounded(...)` / `UNBOUNDED` call, with the same argument
  (`Max::western(SCALES.len() as u32)` for `western(SCALES.length)`).
- `pub fn to_cardinal(value)`, plus `to_cardinal_with(value, &CardinalOptions)`
  when the JS form takes options (same for ordinal and currency).
- The options structs declared with `options! { ... }` (see below).
- `pub static LANGUAGE: Language { code, name, kind: Kind::Full, cardinal, ordinal, currency }`
  using `form!(plain f, MAX)` for a form without options and
  `form!(with_options f_with, Options, MAX)` for one with options. Keep the
  `code` and `name` already in the stub.

**Locale profile** (`Kind::Profile { base }`, JS `variantOf`): identical
numerals to its base, only the default currency differs.
`pub use crate::{base}::{to_cardinal, to_ordinal, CARDINAL_MAX, ...}` (and
`to_cardinal_with` + `CardinalOptions` if the base has them), own
`CurrencyOptions` via `options!` with the profile's default currency,
`to_currency`/`to_currency_with` that build the base's `CurrencyOptions` and
delegate, `pub const VARIANT_OF: &str = "base-CODE"`.

**Bare-tag alias** (`Kind::Alias { target }`, JS `aliasOf`): re-exports the
target's cardinal and ordinal (functions, options types, `*_MAX` constants).
Currency requires an explicit currency: `options!` with `required currency`
first, then the target's other currency options with their defaults;
`pub fn to_currency(value, currency: Currency)` and
`to_currency_with(value, &CurrencyOptions)` delegating to the target;
`pub const ALIAS_OF: &str = "target-CODE"`. If the target's currency form has
no option besides `currency`, the alias `options!` has only the `required`
line.

## JS → Rust mapping

| JavaScript | Rust |
| --- | --- |
| `parseCardinalValue(value)` → `{ isNegative, integerPart, decimalPart }` | `let n = parse_cardinal(&value.into())?;` → `n.negative: bool`, `n.integer: BigUint`, `n.fraction: Option<String>` (digit string as written, `None` when absent; never `Some("")`) |
| `parseOrdinalValue(value)` → bigint | `let n = parse_ordinal(&value.into())?;` → `BigUint` (already positive) |
| `parseCurrencyValue(value, minorUnitDigits(currency))` → `{ isNegative, dollars, cents }` | `let m = parse_currency(&value.into(), currency.minor_unit_digits())?;` → `m.negative`, `m.major: BigUint`, `m.minor: u32` |
| `checkMax(integerPart, cardinalMax, decimalPart)` | `CARDINAL_MAX.check(&n.integer, n.fraction.as_deref())?;` |
| `checkMax(x, max)` | `MAX.check(&x, None)?;` |
| `resolveOptions(options, defaults, values)` | the `options!` struct: fields are already resolved; pass them on |
| `import { xx as CURRENCY_VOCAB } from './utils/currency-vocab.js'` | `use crate::currency::vocab::XX as CURRENCY_VOCAB;` (`EN`, `FR`, `ES`, `PT_BR`, `PT_PT`, `ZH_HANS`, `ZH_HANT`, `SR_CYRL`, `SR_LATN`, `AM`, `AM_LATN`, `HBO`, `HE`, ...) |
| `const { major, minor } = CURRENCY_VOCAB[currency]` | `let words = CURRENCY_VOCAB.words(options.currency)?;` → `words.major: &[&str]`, `words.minor: Option<&[&str]>`, `words.major_gender`, `words.minor_gender` |
| `major[1]` / `major.length < 2 ? major[0] : major[1]` | `words.major_at(1)` — clamps to the last form, so an invariable noun serves every index. Same for `words.minor_at(i)` (panics only when `minor` is `None`, which `assert_minor_unit` rules out before) |
| `assertCurrencyExponent(cents, currency)` | `assert_minor_unit(m.minor, options.currency)?;` |
| `currencyValues.currency` | derived from the table: `currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;` in `options!` |
| `throw new RangeError(...)` for a value | return `Err(...)` with an existing `Error` variant (`OutOfRange`, `InvalidNumber`, `NoMinorUnit`, ...). Never panic on input |
| `n % 1000n`, `n / 1000n` loop | `split_groups(&n, 1000)` → `Vec<u32>` least-significant first (`[]` for zero); `split_groups_indian(&n, 1000, 100)` for 3-2-2 grouping |
| `Number(n)` on a small bigint | `n.to_u64()` (from `num_traits::ToPrimitive`), then `as u32`/`as usize` |
| `n === 0n`, `n === 1n` | `n.is_zero()`, `n.is_one()` (`num_traits::{Zero, One}`) |
| `BigInt(decimalPart.slice(i))` | `digits_to_biguint(&fraction[i..])` |
| `'一'`-style string tables | `const X: [&str; 10] = [...]` or `const X: &[&str] = &[...]` |
| `typedef Gender = 'masculine' \| 'feminine'` | `crate::options::Gender` (shared). Other enums: declare with `option_enum!` in your module |

Imports a full implementation typically needs:

```rust
use crate::Result;
use crate::currency::vocab::XX as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::options;            // and option_enum, Gender when needed
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
```

Unused imports are warnings, and CI runs clippy with `-D warnings`: remove what
you don't use.

## Options

```rust
options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Grammatical gender of the counted noun
        gender: Gender = Gender::Masculine;
        /// Custom word for negative numbers
        negative_word: String = NEGATIVE.to_string();
        /// Use "and" after hundreds
        and: bool = false;
    }
}
```

- Field names are the JS keys in snake_case (`hundredPairing` → `hundred_pairing`,
  `noHundredPairing` → `no_hundred_pairing`, `case` stays `case`). The fixtures
  use these names, so they must match exactly.
- Each field's doc comment is **one line**, copied from the JS `@property`
  description. It becomes the option's description in the generated docs.
- Field types: `bool`, `String`, `Currency`, `Gender`, or an enum you declare:

```rust
option_enum! {
    /// Grammatical case (الإعراب).
    pub enum Case {
        /// Nominative (مرفوع).
        Nominative = "nominative",
        /// Accusative (منصوب).
        Accusative = "accusative",
        /// Genitive (مجرور).
        Genitive = "genitive",
    }
}
```

The string on the right is the JS value (what the fixtures and the CLI use).
A JS enum whose values are not ASCII identifiers (`hundredSpelling: 'مئة' | 'مائة'`)
still gets an ASCII variant name with the original text as its string.

- The `currency` field must name the table: `currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;`.
- `to_currency_with` does, in this order: `let words = CURRENCY_VOCAB.words(options.currency)?;`
  (validates the currency), `parse_currency(&value.into(), options.currency.minor_unit_digits())?`,
  `CURRENCY_MAX.check(&money.major, None)?`, `assert_minor_unit(money.minor, options.currency)?`,
  then build.

## Documentation

`missing_docs` is on. Every `pub` item needs a doc comment; the module needs a
`//!` header translated from the JS file's header comment (conventions, key
features), with a short doctest showing the three forms when practical:

```rust
//! ```
//! use n2words::de_de::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "zweiundvierzig");
//! # Ok::<(), n2words::Error>(())
//! ```
```

Use real outputs from the fixtures in doctests; `cargo test --doc` runs them.

## Faithfulness

Port the logic as written, including comments explaining grammar; do not
"improve" the words. The fixtures are the spec: when the JS and the fixture
disagree, the fixture (which the JS passes) wins — re-read the JS. Edge cases
the JS handles (invariable nouns, `major.length < 2`, special zero/one forms,
hundred pairing, "and" placement) all have fixture cases.

Performance idioms: use `n.to_u64()` fast paths for small numbers like the JS
`Number(n)` fast paths, build with `String::push_str`/`format!`, and avoid
cloning `BigUint`s in loops. Don't over-optimize; clarity first.

## Checking a port

```bash
cargo test --test conversions -- <code>        # fixtures, e.g. `de-DE` (also matches `de`)
cargo test --test contracts -- --skip no_stubs # range / options / alias / profile / currency gates
cargo test --doc                               # doctests (yours; others may fail until ported)
cargo clippy --all-targets -- -D warnings      # must be clean for your files
cargo fmt
```

`no_stubs` fails until every language is ported; ignore it while porting.
Fixture trials are named `{code}::{form}`; a filter is a substring match.

Checklist before calling a language done:

- [ ] stub replaced; `LANGUAGE.kind` is the right `Kind` with the right base/target
- [ ] every form the JS exports is implemented, with its `*_MAX` constant
- [ ] options declared with `options!`, names and defaults as in the JS
- [ ] `tests/conversions` passes for the code (and its alias / profiles, if you ported those)
- [ ] `tests/contracts` passes (all but `no_stubs`)
- [ ] module `//!` header and doc comments on every `pub` item, doctest with real outputs
- [ ] `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt` run
