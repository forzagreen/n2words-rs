//! Fixture-driven conversion tests.
//!
//! `tests/fixtures/{code}.json` holds the cases imported from the JavaScript
//! library (`scripts/import-fixtures.mjs`): for each form, `input` (a JSON
//! number, a string, or `{ "bigint": "..." }`), `expected`, and optional
//! `options` as `{ name: value }`. One trial runs per language and form, so
//! `cargo test --test conversions en-US` filters to a language.
//!
//! Also enforced here, as in the JavaScript suite:
//! - every registered language has a fixture file and vice versa;
//! - every form a language implements has fixtures (≥ 5 cases), and every
//!   fixture form is implemented;
//! - the ordinal form rejects 0, -1 and 1.5.

use libtest_mimic::{Arguments, Failed, Trial};
use n2words::{FormKind, Language, Number, language, languages};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const MIN_CASES_PER_FORM: usize = 5;

#[derive(Deserialize)]
struct Fixture {
    code: String,
    cardinal: Option<Vec<Case>>,
    ordinal: Option<Vec<Case>>,
    currency: Option<Vec<Case>>,
}

impl Fixture {
    fn cases(&self, kind: FormKind) -> Option<&Vec<Case>> {
        match kind {
            FormKind::Cardinal => self.cardinal.as_ref(),
            FormKind::Ordinal => self.ordinal.as_ref(),
            FormKind::Currency => self.currency.as_ref(),
        }
    }
}

#[derive(Deserialize)]
struct Case {
    input: serde_json::Value,
    expected: String,
    #[serde(default)]
    options: serde_json::Map<String, serde_json::Value>,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixtures() -> Vec<Fixture> {
    let mut entries: Vec<_> = fs::read_dir(fixtures_dir())
        .expect("tests/fixtures exists")
        .map(|e| e.expect("readable entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("readable fixture");
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        })
        .collect()
}

fn to_number(value: &serde_json::Value) -> Number<'static> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Number::from(i)
            } else if let Some(u) = n.as_u64() {
                Number::from(u)
            } else {
                Number::from(n.as_f64().expect("a JSON number is an f64"))
            }
        }
        serde_json::Value::String(s) => Number::from(s.clone()),
        serde_json::Value::Object(o) => {
            let digits = o
                .get("bigint")
                .and_then(|v| v.as_str())
                .expect("{ bigint: \"...\" }");
            Number::from(digits.parse::<num_bigint::BigInt>().expect("bigint digits"))
        }
        other => panic!("unsupported fixture input {other}"),
    }
}

fn to_pairs(options: &serde_json::Map<String, serde_json::Value>) -> Vec<(String, String)> {
    options
        .iter()
        .map(|(k, v)| {
            let value = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            (k.clone(), value)
        })
        .collect()
}

fn describe(case: &Case) -> String {
    let mut s = case.input.to_string();
    if !case.options.is_empty() {
        s.push_str(&format!(
            " with {}",
            serde_json::Value::Object(case.options.clone())
        ));
    }
    s
}

fn run_form(lang: &'static Language, kind: FormKind, cases: &[Case]) -> Result<(), Failed> {
    let Some(form) = lang.form(kind) else {
        return Err(format!(
            "{} has {kind} fixtures but does not implement the form",
            lang.code
        )
        .into());
    };
    let mut failures = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let pairs = to_pairs(&case.options);
        let borrowed: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        match form.convert(to_number(&case.input), &borrowed) {
            Ok(actual) if actual == case.expected => {}
            Ok(actual) => failures.push(format!(
                "case {}/{} {}:\n    expected: {:?}\n    actual:   {:?}",
                i + 1,
                cases.len(),
                describe(case),
                case.expected,
                actual
            )),
            Err(e) => failures.push(format!(
                "case {}/{} {} returned an error: {e}\n    expected: {:?}",
                i + 1,
                cases.len(),
                describe(case),
                case.expected
            )),
        }
    }
    if cases.len() < MIN_CASES_PER_FORM {
        failures.push(format!(
            "only {} cases (minimum {MIN_CASES_PER_FORM})",
            cases.len()
        ));
    }
    if kind == FormKind::Ordinal {
        for bad in [Number::from(0), Number::from(-1), Number::from(1.5)] {
            if let Ok(out) = form.convert(bad.clone(), &[]) {
                failures.push(format!("ordinal {bad:?} should be an error, got {out:?}"));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

fn structural_checks(fixtures: &[Fixture]) -> Vec<Trial> {
    let fixture_codes: BTreeSet<String> = fixtures.iter().map(|f| f.code.clone()).collect();
    let language_codes: BTreeSet<String> = languages().iter().map(|l| l.code.to_string()).collect();

    let missing_fixtures: Vec<String> =
        language_codes.difference(&fixture_codes).cloned().collect();
    let missing_languages: Vec<String> =
        fixture_codes.difference(&language_codes).cloned().collect();

    let mut forms_without_fixtures = Vec::new();
    for fixture in fixtures {
        if let Some(lang) = language(&fixture.code) {
            for (kind, _) in lang.forms() {
                if fixture.cases(kind).is_none() {
                    forms_without_fixtures.push(format!(
                        "{} implements {kind} but has no {kind} fixtures",
                        lang.code
                    ));
                }
            }
        }
    }

    vec![
        Trial::test("structure::every_language_has_fixtures", move || {
            if missing_fixtures.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "languages without a fixture file: {}",
                    missing_fixtures.join(", ")
                )
                .into())
            }
        }),
        Trial::test("structure::every_fixture_has_a_language", move || {
            if missing_languages.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "fixture files without a language: {}",
                    missing_languages.join(", ")
                )
                .into())
            }
        }),
        Trial::test(
            "structure::every_implemented_form_has_fixtures",
            move || {
                if forms_without_fixtures.is_empty() {
                    Ok(())
                } else {
                    Err(forms_without_fixtures.join("\n").into())
                }
            },
        ),
    ]
}

fn main() {
    let args = Arguments::from_args();
    let fixtures = load_fixtures();

    let mut trials = structural_checks(&fixtures);
    for fixture in fixtures {
        let Some(lang) = language(&fixture.code) else {
            continue; // reported by the structural check
        };
        for kind in FormKind::ALL {
            let Some(cases) = fixture.cases(kind) else {
                continue;
            };
            let cases: Vec<Case> = cases
                .iter()
                .map(|c| Case {
                    input: c.input.clone(),
                    expected: c.expected.clone(),
                    options: c.options.clone(),
                })
                .collect();
            trials.push(Trial::test(format!("{}::{kind}", lang.code), move || {
                run_form(lang, kind, &cases)
            }));
        }
    }

    libtest_mimic::run(&args, trials).exit();
}
