//! Generates `LANGUAGES.md` from the registry, so the docs can't drift from
//! the code. Run `cargo run --example generate_languages_md` and commit the
//! result; CI checks that the committed file is current.

use n2words::options::OptionKind;
use n2words::{Currency, FormKind, Kind, Language, Max, languages};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

fn anchor(lang: &Language) -> String {
    let text = format!("{} {}", lang.name, lang.code).to_lowercase();
    let slug: String = text
        .chars()
        .filter_map(|c| match c {
            c if c.is_alphanumeric() => Some(c),
            ' ' | '-' => Some('-'),
            _ => None,
        })
        .collect();
    slug.trim_matches('-').to_string()
}

fn ceiling(max: Max) -> String {
    match max {
        Max::Pow10(e) => format!("10^{e} - 1"),
        Max::Unbounded => "∞".to_string(),
    }
}

fn has_options(lang: &Language) -> bool {
    lang.forms().any(|(_, f)| !f.options.is_empty())
}

fn main() {
    let all = languages();
    let full: Vec<&Language> = all.iter().copied().filter(|l| !l.is_alias()).collect();
    let aliases: Vec<&Language> = all.iter().copied().filter(|l| l.is_alias()).collect();

    // Family: primary subtag -> (display name, variants).
    let mut families: BTreeMap<String, Vec<&Language>> = BTreeMap::new();
    for lang in &full {
        families
            .entry(lang.primary_subtag().to_string())
            .or_default()
            .push(lang);
    }
    let alias_of: BTreeMap<&str, &Language> =
        aliases.iter().map(|a| (a.primary_subtag(), *a)).collect();
    let family_name = |subtag: &str, variants: &[&Language]| -> String {
        alias_of
            .get(subtag)
            .map(|a| a.name.to_string())
            .unwrap_or_else(|| {
                // No alias: strip the parenthesised region from the first variant.
                let name = variants[0].name;
                name.split(" (")
                    .next()
                    .unwrap_or(name)
                    .replace("Simplified ", "")
                    .replace("Brazilian ", "")
                    .replace("European ", "")
            })
    };

    let mut md = String::new();
    let count = |kind: FormKind| full.iter().filter(|l| l.form(kind).is_some()).count();
    writeln!(md, "# Supported Languages\n").unwrap();
    writeln!(md, "> **Auto-generated** — do not edit by hand. Run `cargo run --example generate_languages_md` to update.\n").unwrap();
    writeln!(
        md,
        "n2words supports **{} languages** ({} regional/script variants total) with\ncardinal number conversion, {} variants with ordinal support, {} variants\nwith currency support.\n",
        families.len(),
        full.len(),
        count(FormKind::Ordinal),
        count(FormKind::Currency)
    )
    .unwrap();
    writeln!(
        md,
        "{} languages have a **bare-tag entry point** that resolves without a region\nsubtag (`n2words::de`); the other {} require a specific variant because their\nvariants genuinely diverge in script or core numbering grammar.\n",
        aliases.len(),
        families.len() - aliases.len()
    )
    .unwrap();
    writeln!(md, "Language codes follow [IETF BCP 47](https://tools.ietf.org/html/bcp47); the Rust module is the code in snake_case (`en_us`, `zh_hans_cn`).\n").unwrap();

    writeln!(md, "## Languages\n").unwrap();
    writeln!(md, "|Entry point|Language|Variants|").unwrap();
    writeln!(md, "|-----------|--------|--------|").unwrap();
    let mut rows: Vec<(String, String)> = families
        .iter()
        .map(|(subtag, variants)| {
            let name = family_name(subtag, variants);
            let entry = alias_of.get(subtag.as_str()).map_or("—".to_string(), |a| {
                let Kind::Alias { target } = a.kind else {
                    unreachable!()
                };
                let target = all
                    .iter()
                    .find(|l| l.code == target)
                    .expect("alias target registered");
                format!("[`{}`](#{})", a.code, anchor(target))
            });
            let variants_md: Vec<String> = variants
                .iter()
                .map(|v| {
                    let default = match alias_of.get(subtag.as_str()) {
                        Some(a)
                            if matches!(a.kind, Kind::Alias { target } if target == v.code)
                                && variants.len() > 1 =>
                        {
                            " (default)"
                        }
                        _ => "",
                    };
                    format!("[`{}`](#{}){default}", v.code, anchor(v))
                })
                .collect();
            (
                name.clone(),
                format!("|{entry}|{name}|{}|", variants_md.join(", ")),
            )
        })
        .collect();
    rows.sort();
    for (_, row) in rows {
        writeln!(md, "{row}").unwrap();
    }
    writeln!(md, "\n`Entry point` is `—` for the languages with no safe default variant. Where a\nfamily has more than one variant, `(default)` marks the one its entry point\nresolves to.\n").unwrap();

    writeln!(md, "## Usage\n").unwrap();
    writeln!(md, "```rust\nuse n2words::{{de, en_gb, Currency}};\n\nde::to_cardinal(42)?;                 // \"zweiundvierzig\"\nde::to_ordinal(42)?;                  // \"zweiundvierzigste\"\nde::to_currency(42.5, Currency::EUR)?; // a bare tag names the currency explicitly\nen_gb::to_currency(42.5)?;            // a region-qualified code has a default\n```\n").unwrap();

    writeln!(md, "## All Regional Variants\n").unwrap();
    writeln!(md, "Per-variant detail: the largest value each form converts, with a `*` where the\nvariant declares options (click through to the options table).\n").unwrap();
    writeln!(md, "|Code|Language|Cardinal|Ordinal|Currency|").unwrap();
    writeln!(md, "|----|--------|:------:|:-----:|:------:|").unwrap();
    for lang in &full {
        let cell = |kind: FormKind| match lang.form(kind) {
            None => String::new(),
            Some(f) if f.options.is_empty() => ceiling(f.max),
            Some(f) => format!("{} [*](#{})", ceiling(f.max), anchor(lang)),
        };
        writeln!(
            md,
            "|`{}`|{}|{}|{}|{}|",
            lang.code,
            lang.name,
            cell(FormKind::Cardinal),
            cell(FormKind::Ordinal),
            cell(FormKind::Currency)
        )
        .unwrap();
    }
    writeln!(md, "\nEach form column shows the largest value it converts (`10^N - 1`), `∞` when unbounded, or blank when the form isn't supported.\n").unwrap();

    // Currency coverage.
    let mut defaults: BTreeMap<Currency, Vec<&str>> = BTreeMap::new();
    let mut also: BTreeMap<Currency, Vec<&str>> = BTreeMap::new();
    for lang in &full {
        let Some(form) = lang.form(FormKind::Currency) else {
            continue;
        };
        let Some(spec) = form.option("currency") else {
            continue;
        };
        let default = spec.default_value().unwrap_or_default();
        if let OptionKind::Currency(table) = spec.kind {
            for c in table.codes() {
                if c.code() == default {
                    defaults.entry(c).or_default().push(lang.code);
                } else {
                    let family = alias_of
                        .get(lang.primary_subtag())
                        .map_or(lang.code, |a| a.code);
                    let list = also.entry(c).or_default();
                    if !list.contains(&family) {
                        list.push(family);
                    }
                }
            }
        }
    }
    let mut currencies: Vec<Currency> = defaults.keys().chain(also.keys()).copied().collect();
    currencies.sort();
    currencies.dedup();
    let pairs: usize = full
        .iter()
        .filter_map(|l| l.form(FormKind::Currency))
        .filter_map(|f| f.option("currency"))
        .filter_map(|s| match s.kind {
            OptionKind::Currency(t) => Some(t.codes().count()),
            _ => None,
        })
        .sum();
    writeln!(md, "## Currency Coverage\n").unwrap();
    writeln!(
        md,
        "Which languages can name which currency. {} variants name {} distinct\nISO 4217 currencies, across {} language/currency pairs. A language always\nnames its own default currency; anything in the third column is opt-in via the\n`currency` option, and asking for a currency a language has no words for is an\n`Error::UnsupportedCurrency`, never a guess.\n",
        count(FormKind::Currency),
        currencies.len(),
        pairs
    )
    .unwrap();
    writeln!(md, "|Currency|Default for|Also names it|").unwrap();
    writeln!(md, "|--------|-----------|-------------|").unwrap();
    let fmt_codes = |codes: Option<&Vec<&str>>| {
        codes
            .filter(|c| !c.is_empty())
            .map_or("—".to_string(), |c| {
                c.iter()
                    .map(|x| format!("`{x}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
    };
    for c in currencies {
        writeln!(
            md,
            "|`{}`|{}|{}|",
            c.code(),
            fmt_codes(defaults.get(&c)),
            fmt_codes(also.get(&c))
        )
        .unwrap();
    }

    // Options.
    let with_options: Vec<&Language> = full.iter().copied().filter(|l| has_options(l)).collect();
    writeln!(md, "\n## Language Options\n").unwrap();
    writeln!(
        md,
        "{} variants take options through the `to_<form>_with` functions and their\n`<Form>Options` structs (whose `Default` is the language's defaults). Every option\nbelongs to exactly one form. In the dynamic API and the CLI the same options are\naddressed by name (`--gender feminine`, `--no-and`).\n",
        with_options.len()
    )
    .unwrap();
    let mut sorted = with_options.clone();
    sorted.sort_by_key(|l| l.name);
    for lang in sorted {
        writeln!(md, "### {} (`{}`)\n", lang.name, lang.code).unwrap();
        writeln!(md, "|Option|Form|Type|Default|Description|").unwrap();
        writeln!(md, "|------|----|----|-------|-----------|").unwrap();
        for (kind, form) in lang.forms() {
            for spec in form.options {
                let ty = match spec.kind {
                    OptionKind::Bool => "`boolean`".to_string(),
                    OptionKind::Text => "`string`".to_string(),
                    _ => spec.type_display().replace('|', "\\|"),
                };
                writeln!(
                    md,
                    "|`{}`|{}|{}|`{}`|{}|",
                    spec.name,
                    kind.name(),
                    ty,
                    spec.default_value().unwrap_or_else(|| "required".into()),
                    spec.description()
                )
                .unwrap();
            }
        }
        writeln!(md).unwrap();
    }

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("LANGUAGES.md");
    std::fs::write(&path, md.trim_end().to_string() + "\n").expect("LANGUAGES.md written");
    println!("wrote {}", path.display());
}
