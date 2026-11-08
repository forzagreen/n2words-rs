//! Behavioural gates every language must clear, ported from the JavaScript
//! suite. They know nothing about any particular language: each walks the
//! registry and checks declarations against behaviour.
//!
//! - **range**: each form is well-formed and injective across every power of
//!   ten below its ceiling, refuses exactly the ceiling, and accepts the value
//!   just below it.
//! - **options**: declared defaults reproduce the no-options output, an
//!   unknown option is rejected, every allowed enum value is accepted and an
//!   out-of-set one is rejected.
//! - **alias**: a bare-tag alias forwards cardinal/ordinal untouched and needs
//!   an explicit currency.
//! - **profile**: a locale profile matches its base's numerals and its
//!   currency equals the base's with the profile's default named.
//! - **currency vocabulary**: every currency a language advertises round-trips
//!   at its own precision (3 decimals for the dinars; no fraction for the yen).
//! - **no stubs**: every registered language implements at least one form.

use n2words::options::OptionKind;
use n2words::{Currency, Error, Form, FormKind, Kind, Language, Max, Number, language, languages};
use num_bigint::BigUint;
use num_traits::One;

fn pow10(e: u32) -> BigUint {
    num_traits::pow(BigUint::from(10u32), e as usize)
}

fn well_formed(s: &str) -> bool {
    !s.is_empty()
        && s == s.trim()
        && !s.contains("  ")
        && !s.contains("undefined")
        && !s.contains("NaN")
}

/// Options that make a form callable: required options get their first allowed value.
fn baseline(form: &Form) -> Vec<(&'static str, String)> {
    form.options
        .iter()
        .filter(|o| o.is_required())
        .map(|o| {
            (
                o.name,
                o.values().expect("required options have a fixed set")[0].to_string(),
            )
        })
        .collect()
}

fn borrow<'a>(pairs: &'a [(&'static str, String)]) -> Vec<(&'a str, &'a str)> {
    pairs.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

fn convert(
    form: &Form,
    value: impl Into<Number<'static>>,
    extra: &[(&str, &str)],
) -> Result<String, Error> {
    let base = baseline(form);
    let mut pairs = borrow(&base);
    pairs.extend(extra.iter().copied());
    form.convert(value, &pairs)
}

/// A registered language with no form yet; `no_stubs` reports these.
fn is_stub(lang: &Language) -> bool {
    lang.forms().next().is_none()
}

fn report(gate: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{gate} gate failed:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn range_contract() {
    let mut failures = Vec::new();
    for lang in languages() {
        for (kind, form) in lang.forms() {
            let label = format!("{} {kind}", lang.code);
            let top = match form.max {
                Max::Pow10(e) => e,
                Max::Unbounded => 330,
            };
            let mut seen: Vec<String> = Vec::new();
            for e in 1..top {
                match convert(form, pow10(e), &[]) {
                    Ok(out) if !well_formed(&out) => {
                        failures.push(format!("{label}: 10^{e} malformed: {out:?}"))
                    }
                    Ok(out) if seen.contains(&out) => {
                        failures.push(format!("{label}: magnitude collapse, 10^{e} spells the same as a lower power: {out:?}"));
                    }
                    Ok(out) => seen.push(out),
                    Err(err) => failures.push(format!("{label}: 10^{e} failed: {err}")),
                }
            }
            if let Max::Pow10(e) = form.max {
                let ceiling = pow10(e);
                match convert(form, ceiling.clone(), &[]) {
                    Err(Error::OutOfRange { max }) if max == form.max => {}
                    Err(err) => failures.push(format!(
                        "{label}: at the ceiling, expected OutOfRange, got {err:?}"
                    )),
                    Ok(out) => failures.push(format!(
                        "{label}: did not refuse its declared ceiling 10^{e}: {out:?}"
                    )),
                }
                match convert(form, &ceiling - BigUint::one(), &[]) {
                    Ok(out) if well_formed(&out) => {}
                    Ok(out) => failures.push(format!(
                        "{label}: malformed just below the ceiling: {out:?}"
                    )),
                    Err(err) => failures.push(format!(
                        "{label}: refused 10^{e} - 1, just below its ceiling: {err}"
                    )),
                }
            }
        }
    }
    report("range", failures);
}

#[test]
fn options_contract() {
    let mut failures = Vec::new();
    for lang in languages() {
        for (kind, form) in lang.forms() {
            if form.options.is_empty() {
                // A form without options must reject any.
                if convert(form, 101, &[("bogus", "1")]).is_ok() {
                    failures.push(format!(
                        "{} {kind}: accepted an option although it declares none",
                        lang.code
                    ));
                }
                continue;
            }
            let label = format!("{} {kind}", lang.code);
            // A whole number: some currencies (JPY, ...) reject fractions by design.
            let sample = if kind == FormKind::Currency { 42 } else { 101 };

            // Declared defaults really are the defaults.
            let defaults: Vec<(&str, String)> = form
                .options
                .iter()
                .filter_map(|o| o.default_value().map(|d| (o.name, d)))
                .collect();
            let defaults_ref: Vec<(&str, &str)> =
                defaults.iter().map(|(k, v)| (*k, v.as_str())).collect();
            match (
                convert(form, sample, &[]),
                convert(form, sample, &defaults_ref),
            ) {
                (Ok(a), Ok(b)) if a == b => {}
                (a, b) => failures.push(format!(
                    "{label}: explicit defaults differ from no options: {a:?} vs {b:?}"
                )),
            }

            // Malformed options fail loudly.
            if !matches!(
                convert(form, sample, &[("__unknown__", "x")]),
                Err(Error::UnknownOption { .. })
            ) {
                failures.push(format!(
                    "{label}: an unknown option was not rejected with UnknownOption"
                ));
            }

            for spec in form.options {
                let Some(values) = spec.values() else {
                    continue; // free text: anything goes
                };
                if let Some(default) = spec.default_value() {
                    if !values.contains(&default.as_str()) {
                        failures.push(format!(
                            "{label}: default {default:?} of {} is not in its allowed set",
                            spec.name
                        ));
                    }
                }
                for value in &values {
                    if let Err(err) = convert(form, sample, &[(spec.name, value)]) {
                        failures.push(format!(
                            "{label}: declared value {value:?} of {} was rejected: {err}",
                            spec.name
                        ));
                    }
                }
                let probe = if matches!(spec.kind, OptionKind::Bool) {
                    "maybe"
                } else {
                    "__not_in_set__"
                };
                match convert(form, sample, &[(spec.name, probe)]) {
                    Err(Error::InvalidOptionValue { key, .. }) if key == spec.name => {}
                    other => failures.push(format!(
                        "{label}: out-of-set {} = {probe:?} gave {other:?}",
                        spec.name
                    )),
                }
            }
        }
    }
    report("options", failures);
}

const SAMPLES: &[i64] = &[
    1,
    2,
    3,
    7,
    11,
    21,
    42,
    99,
    100,
    101,
    111,
    1000,
    1001,
    12_345,
    123_456,
    1_000_000,
    999_999_999,
];

fn same_numerals(label: &str, a: &Language, b: &Language, failures: &mut Vec<String>) {
    for kind in [FormKind::Cardinal, FormKind::Ordinal] {
        match (a.form(kind), b.form(kind)) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => {
                failures.push(format!("{label}: {kind} is implemented on only one side"))
            }
            (Some(fa), Some(fb)) => {
                let mut values: Vec<Number<'static>> =
                    SAMPLES.iter().map(|&v| Number::from(v)).collect();
                if kind == FormKind::Cardinal {
                    values.extend([
                        Number::from(0),
                        Number::from(-5),
                        Number::from("3.14"),
                        Number::from("-0.07"),
                    ]);
                }
                for value in values {
                    let (ra, rb) = (
                        convert(fa, value.clone(), &[]),
                        convert(fb, value.clone(), &[]),
                    );
                    if ra != rb {
                        failures.push(format!(
                            "{label}: {kind} of {value:?} differs: {ra:?} vs {rb:?}"
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn alias_contract() {
    let mut failures = Vec::new();
    for lang in languages() {
        let Kind::Alias { target } = lang.kind else {
            continue;
        };
        let target = language(target).expect("alias target is registered");
        if is_stub(lang) || is_stub(target) {
            continue;
        }
        if target.is_alias() {
            failures.push(format!(
                "{}: alias of another alias {}",
                lang.code, target.code
            ));
        }
        same_numerals(
            &format!("{} vs {}", lang.code, target.code),
            lang,
            target,
            &mut failures,
        );
        match (lang.currency.as_ref(), target.currency.as_ref()) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => failures.push(format!(
                "{}: currency implemented on only one side",
                lang.code
            )),
            (Some(alias_form), Some(target_form)) => {
                if !matches!(
                    alias_form.convert(42.5, &[]),
                    Err(Error::MissingOption { key: "currency" })
                ) {
                    failures.push(format!(
                        "{}: currency without an explicit currency must fail with MissingOption",
                        lang.code
                    ));
                }
                let Some(default) = target_form
                    .option("currency")
                    .and_then(|o| o.default_value())
                else {
                    failures.push(format!("{}: target has no default currency", target.code));
                    continue;
                };
                for value in ["42.50", "1", "0.01", "1000000"] {
                    let a = alias_form.convert(value, &[("currency", &default)]);
                    let b = target_form.convert(value, &[]);
                    if a != b {
                        failures.push(format!("{}: currency of {value} with {default} differs from {}: {a:?} vs {b:?}", lang.code, target.code));
                    }
                }
            }
        }
    }
    report("alias", failures);
}

#[test]
fn profile_contract() {
    let mut failures = Vec::new();
    for lang in languages() {
        let Kind::Profile { base } = lang.kind else {
            continue;
        };
        let base = language(base).expect("profile base is registered");
        if is_stub(lang) || is_stub(base) {
            continue;
        }
        if base.kind != Kind::Full {
            failures.push(format!(
                "{}: base {} is not a full implementation",
                lang.code, base.code
            ));
        }
        same_numerals(
            &format!("{} vs {}", lang.code, base.code),
            lang,
            base,
            &mut failures,
        );
        let (Some(profile_form), Some(base_form)) =
            (lang.currency.as_ref(), base.currency.as_ref())
        else {
            failures.push(format!("{}: a profile exists to change the default currency, so both it and {} need the form", lang.code, base.code));
            continue;
        };
        let Some(default) = profile_form
            .option("currency")
            .and_then(|o| o.default_value())
        else {
            failures.push(format!("{}: no default currency declared", lang.code));
            continue;
        };
        let base_default = base_form.option("currency").and_then(|o| o.default_value());
        if base_default.as_deref() == Some(default.as_str()) {
            failures.push(format!("{}: same default currency as its base {} ({default}), so it should be an alias, not a profile", lang.code, base.code));
        }
        for value in ["42.50", "1", "0.01", "1000000", "-7.25"] {
            let a = profile_form.convert(value, &[]);
            let b = base_form.convert(value, &[("currency", &default)]);
            if a != b {
                failures.push(format!(
                    "{}: currency of {value} differs from {} with {default}: {a:?} vs {b:?}",
                    lang.code, base.code
                ));
            }
        }
    }
    report("profile", failures);
}

#[test]
fn currency_vocabulary_contract() {
    let mut failures = Vec::new();
    for lang in languages() {
        let Some(form) = lang.currency.as_ref() else {
            continue;
        };
        let Some(spec) = form.option("currency") else {
            failures.push(format!(
                "{}: currency form declares no `currency` option",
                lang.code
            ));
            continue;
        };
        let OptionKind::Currency(table) = spec.kind else {
            failures.push(format!(
                "{}: the `currency` option must be declared with `table = ...`",
                lang.code
            ));
            continue;
        };
        for currency in table.codes() {
            let code = currency.code();
            let label = format!("{} {code}", lang.code);
            let run = |value: &str| form.convert(value, &[("currency", code)]);
            match currency.exponent() {
                0 => {
                    if !matches!(run("1.5"), Err(Error::NoMinorUnit { .. })) {
                        failures.push(format!(
                            "{label}: has no minor unit, so 1.5 must fail with NoMinorUnit"
                        ));
                    }
                    if let Err(err) = run("1") {
                        failures.push(format!("{label}: whole amount failed: {err}"));
                    }
                }
                3 => {
                    let outs: Vec<_> = ["1.001", "1.010", "1.100", "1.500"]
                        .iter()
                        .map(|v| run(v))
                        .collect();
                    if outs.iter().any(Result::is_err) {
                        failures.push(format!(
                            "{label}: a thousandth-unit amount failed: {outs:?}"
                        ));
                    } else {
                        let mut distinct: Vec<&String> =
                            outs.iter().map(|o| o.as_ref().unwrap()).collect();
                        distinct.dedup();
                        if distinct.len() != outs.len() {
                            failures.push(format!("{label}: thousandth-unit amounts collide, so the parser is not keeping 3 digits: {outs:?}"));
                        }
                    }
                }
                _ => {
                    let (a, b) = (run("1.01"), run("1.10"));
                    if a.is_err() || b.is_err() || a == b {
                        failures.push(format!(
                            "{label}: 1.01 and 1.10 must both convert and differ: {a:?} vs {b:?}"
                        ));
                    }
                }
            }
        }
        // A currency the language has no words for is refused, never guessed.
        if let Some(missing) = Currency::ALL.iter().find(|c| !table.contains(**c)) {
            if !matches!(
                form.convert("1", &[("currency", missing.code())]),
                Err(Error::UnsupportedCurrency { .. })
            ) {
                failures.push(format!("{}: {missing} is not in its table and must be refused with UnsupportedCurrency", lang.code));
            }
        }
    }
    report("currency vocabulary", failures);
}

#[test]
fn no_stubs() {
    let stubs: Vec<&str> = languages()
        .iter()
        .filter(|l| l.forms().next().is_none())
        .map(|l| l.code)
        .collect();
    assert!(
        stubs.is_empty(),
        "languages with no form implemented: {}",
        stubs.join(", ")
    );
}
