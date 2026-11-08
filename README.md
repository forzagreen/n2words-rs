# n2words

[![CI](https://github.com/forzagreen/n2words-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/forzagreen/n2words-rs/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/coveralls/github/forzagreen/n2words-rs)](https://coveralls.io/github/forzagreen/n2words-rs)
[![crates.io](https://img.shields.io/crates/v/n2words)](https://crates.io/crates/n2words)
[![docs.rs](https://img.shields.io/docsrs/n2words)](https://docs.rs/n2words)
[![License: MIT](https://img.shields.io/crates/l/n2words)](LICENSE)

**Numbers to words. 50 languages, 72 regional variants. Arbitrary precision.**

The Rust port of [n2words](https://github.com/forzagreen/n2words), the
JavaScript library, with the same languages, the same words and the same
contracts, adapted to Rust: typed per-language option structs, `Result`
instead of exceptions, and `BigInt`-exact input through strings and
[`num-bigint`](https://crates.io/crates/num-bigint).

## Why n2words?

- **Plain functions**: each language is a module of free functions. No
  builders to configure, no global state, no `unsafe`.
- **Three forms**: cardinal ("forty-two"), ordinal ("forty-second") and
  currency ("forty-two dollars and fifty cents").
- **50 languages, 72 regional variants**: European, Asian, Middle Eastern,
  African, and more, most reachable through a bare language tag (`n2words::de`).
- **Exact**: input as any integer, float, numeric string or `BigInt`, so large
  values keep full precision; every form declares the largest value it spells
  and refuses anything past it instead of inventing words.
- **Tiny dependency footprint**: `num-bigint` and `num-traits` only.
- **A CLI too**: `cargo install n2words` ships an `n2words` command whose flags
  are derived from each language's own options.

## Quick start

```bash
cargo add n2words
```

```rust
use n2words::{en, es};

assert_eq!(en::to_cardinal(42)?, "forty-two");
assert_eq!(es::to_cardinal(42)?, "cuarenta y dos");
# Ok::<(), n2words::Error>(())
```

## Forms

| Form     | Function                       | Example                             |
| -------- | ------------------------------ | ----------------------------------- |
| Cardinal | `to_cardinal(42)`              | "forty-two"                         |
| Ordinal  | `to_ordinal(42)`               | "forty-second"                      |
| Currency | `to_currency(42.50)`           | "forty-two dollars and fifty cents" |

```rust
use n2words::en_us::{to_cardinal, to_currency, to_ordinal};

assert_eq!(to_cardinal(1234)?, "one thousand two hundred thirty-four");
assert_eq!(to_ordinal(1234)?, "one thousand two hundred thirty-fourth");
assert_eq!(to_currency(1234.56)?, "one thousand two hundred thirty-four dollars and fifty-six cents");
# Ok::<(), n2words::Error>(())
```

Currency is the one form a bare tag won't guess at. `en` names a *language*,
and a default currency belongs to a *country*, so `n2words::en` takes the
currency explicitly while a region-qualified module has its own default:

```rust
use n2words::{en, en_us, Currency};

assert_eq!(en::to_currency(42.5, Currency::GBP)?, "forty-two pounds and fifty pence");
assert_eq!(en_us::to_currency(42.5)?, "forty-two dollars and fifty cents");
# Ok::<(), n2words::Error>(())
```

A language can name any currency it has words for, not just its own default,
and asking for one it lacks is an `Error::UnsupportedCurrency`, never a guess:

```rust
use n2words::{pt_br, Currency};

assert_eq!(pt_br::to_currency(42.5)?, "quarenta e dois reais e cinquenta centavos");
let eur = pt_br::CurrencyOptions { currency: Currency::EUR, ..Default::default() };
assert_eq!(pt_br::to_currency_with(42.5, &eur)?, "quarenta e dois euros e cinquenta centavos");
assert!(matches!(
    pt_br::to_currency_with(42.5, &pt_br::CurrencyOptions { currency: Currency::CAD, ..Default::default() }),
    Err(n2words::Error::UnsupportedCurrency { .. })
));
# Ok::<(), n2words::Error>(())
```

Each language implements one or more forms; [LANGUAGES.md](LANGUAGES.md) lists
per-language coverage, ceilings and options.

> **Range:** each form spells values up to the largest scale word it knows,
> then returns `Error::OutOfRange` rather than inventing vocabulary. The
> ceiling varies by language *and* form (`es_es` cardinals reach 10^30 − 1,
> ordinals only 10^9 − 1) and is exported as a constant (`en_us::CARDINAL_MAX`).
> Cardinal and currency accept negatives and decimals; ordinal is positive
> integers only.

## Input

Every function takes `impl Into<Number>`: any primitive integer, `f64`/`f32`,
a numeric `&str`/`String` (arbitrary precision, scientific notation accepted),
or a `num_bigint::BigInt`/`BigUint`:

```rust
use n2words::en;

assert_eq!(en::to_cardinal("12345678901234567890")?, "twelve quintillion three hundred forty-five quadrillion six hundred seventy-eight trillion nine hundred one billion two hundred thirty-four million five hundred sixty-seven thousand eight hundred ninety");
assert_eq!(en::to_cardinal(1e21)?, "one sextillion");
assert_eq!(en::to_cardinal(-3.14)?, "minus three point fourteen");
# Ok::<(), n2words::Error>(())
```

## Options

A form with options has a `to_<form>_with(value, &options)` variant taking a
struct whose `Default` is the language's own defaults:

```rust
use n2words::en_us::{self, CardinalOptions};

assert_eq!(en_us::to_cardinal(1501)?, "one thousand five hundred one");
let informal = CardinalOptions { and: true, hundred_pairing: true, ..Default::default() };
assert_eq!(en_us::to_cardinal_with(1501, &informal)?, "fifteen hundred and one");
# Ok::<(), n2words::Error>(())
```

Enum-valued options are Rust enums (`Gender::Feminine`, `Currency::EUR`), so
an invalid value is a compile error rather than a runtime one. The dynamic API
below and the CLI address the same options by name.

## Picking a language at runtime

`language(code)` looks an entry point up by BCP 47 code (any casing, `-` or
`_`) and exposes the same conversions through string-keyed options, with each
form's ceiling and option metadata attached:

```rust
use n2words::{language, FormKind};

let lang = language("fr-BE").expect("Belgian French is supported");
assert_eq!(lang.convert(FormKind::Cardinal, 92, &[])?, "nonante-deux");
assert_eq!(lang.convert(FormKind::Currency, "42.50", &[("currency", "EUR")])?, "quarante-deux euros et cinquante centimes");
for (form, spec) in lang.forms() {
    println!("{form}: up to {}", spec.max.largest_display());
}
# Ok::<(), n2words::Error>(())
```

## CLI

```bash
cargo install n2words
```

```bash
n2words 42 --lang en                   # forty-two
n2words 42 -l fr-FR --ordinal          # quarante-deuxième
n2words 42.50 -l en-US --currency EUR  # forty-two euro and fifty cents
n2words 101 -l es-ES --gender feminine # ciento una
```

`--lang` takes any entry point the crate exports, in any casing. `--cardinal`
(the default), `--ordinal` and `--currency` pick the form; `--currency <CODE>`
names the currency and selects the form in one flag, while `--form currency`
keeps the locale's own default.

**Options are the language's own.** Every flag past the built-ins comes from
the language and form you selected, so the CLI offers exactly what they
declare: `--gender` for Spanish, `--formal` for Chinese, `--and`/`--no-and` for
English currency. Booleans also accept `--flag=false`, and
`--option key=value` reaches any option by its declared name.

```bash
n2words --list               # every entry point, its kind, and the forms it supports
n2words --help --lang es-ES  # es-ES's forms, their ceilings, and its options
```

With no values it reads stdin, one per line, and `--json` emits one record per
line (`{input, output, lang, form, options}`, or `{input, error}`):

```bash
printf '1\n2\n3\n' | n2words -l de
seq 1 100 | n2words -l fr --json > numbers.jsonl
```

Exit codes: `0` success, `1` a usage error (unknown flag, unknown language,
missing `--lang`), `2` at least one value could not be converted. A bad line
in a pipe doesn't stop the rest.

## Supported languages

See **[LANGUAGES.md](LANGUAGES.md)** for the complete list with codes,
ceilings and options.

Highlights: Arabic, Chinese (Simplified/Traditional), English, French, German,
Hindi, Japanese, Korean, Portuguese, Russian, Spanish, and many more.

## Relation to the JavaScript library

Same languages, same test fixtures, same words. What changed is what Rust
changes: `toCardinal(value, options?)` became `to_cardinal(value)` plus
`to_cardinal_with(value, &options)`; `TypeError`/`RangeError` became the
`Error` enum; `cardinalMax` became `CARDINAL_MAX: Max`; option names are
snake_case (`hundred_pairing`). The fixtures in `tests/fixtures` are imported
from the JavaScript repository verbatim, so the two libraries are checked
against the same expectations.

## Contributing

Bug reports, new languages and corrections from native speakers are welcome.
See [CONTRIBUTING.md](CONTRIBUTING.md), and
[docs/porting.md](docs/porting.md) for how a language file is structured.

## License

[MIT](LICENSE) © Wael TELLAT & contributors
