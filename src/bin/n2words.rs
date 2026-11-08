//! The `n2words` command-line tool: spell numbers from the shell.
//!
//! Nothing about a language is hardcoded here. Flags, help text and
//! validation come from each language's own declarations in the registry
//! ([`n2words::language`]), so adding a language or an option to the crate
//! gives the CLI a new flag with no edit in this file.
//!
//! ```text
//! n2words 42 --lang en                   # forty-two
//! n2words 42 -l fr-FR --ordinal          # quarante-deuxième
//! n2words 42.50 -l en-US --currency EUR  # forty-two euro and fifty cents
//! n2words 101 -l es-ES --gender feminine # ciento una
//! n2words --list                         # every entry point and its forms
//! n2words --help --lang es-ES            # es-ES's forms, ceilings and options
//! seq 1 100 | n2words -l de --json       # one JSON record per line
//! ```
//!
//! Exit codes: `0` success, `1` usage error, `2` at least one value could not
//! be converted.

use n2words::options::{OptionKind, OptionSpec};
use n2words::{Error, Form, FormKind, Kind, Language, language, languages};
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

const EXIT_USAGE: u8 = 1;
const EXIT_CONVERSION: u8 = 2;

/// An error in how the CLI was invoked, as opposed to a conversion failure.
struct UsageError {
    message: String,
    hint: Option<String>,
}

impl UsageError {
    fn new(message: impl Into<String>) -> Self {
        UsageError {
            message: message.into(),
            hint: None,
        }
    }

    fn with_hint(message: impl Into<String>, hint: impl Into<String>) -> Self {
        UsageError {
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

/// The built-in flags, parsed in a first lenient pass.
#[derive(Default)]
struct Globals {
    lang: Option<String>,
    form: Option<String>,
    cardinal: bool,
    ordinal: bool,
    currency: Option<String>,
    options: Vec<String>,
    json: bool,
    list: bool,
    help: bool,
    version: bool,
    /// Everything else: language flags and positional values, in order.
    rest: Vec<String>,
}

/// A token that is a (possibly negative, possibly fractional) number rather
/// than a flag.
fn looks_numeric(token: &str) -> bool {
    let body = token
        .strip_prefix('-')
        .or_else(|| token.strip_prefix('+'))
        .unwrap_or(token);
    body.starts_with(|c: char| c.is_ascii_digit() || c == '.')
}

fn parse_globals(args: &[String]) -> Result<Globals, UsageError> {
    let mut g = Globals::default();
    let mut i = 0;
    let value = |i: &mut usize, flag: &str| -> Result<String, UsageError> {
        *i += 1;
        args.get(*i)
            .cloned()
            .ok_or_else(|| UsageError::new(format!("Flag {flag} needs a value")))
    };
    while i < args.len() {
        let arg = args[i].as_str();
        // `--flag=value` for the built-ins that take a value.
        let (name, inline) = match arg.split_once('=') {
            Some((n, v)) if arg.starts_with("--") => (n, Some(v.to_string())),
            _ => (arg, None),
        };
        match name {
            "--lang" | "-l" => {
                g.lang = Some(match inline {
                    Some(v) => v,
                    None => value(&mut i, name)?,
                })
            }
            "--form" | "-f" => {
                g.form = Some(match inline {
                    Some(v) => v,
                    None => value(&mut i, name)?,
                })
            }
            "--currency" => {
                g.currency = Some(match inline {
                    Some(v) => v,
                    None => value(&mut i, name)?,
                })
            }
            "--option" => g.options.push(match inline {
                Some(v) => v,
                None => value(&mut i, name)?,
            }),
            "--cardinal" => g.cardinal = true,
            "--ordinal" => g.ordinal = true,
            "--json" | "-j" => g.json = true,
            "--list" => g.list = true,
            "--help" | "-h" => g.help = true,
            "--version" | "-v" => g.version = true,
            _ => g.rest.push(arg.to_string()),
        }
        i += 1;
    }
    Ok(g)
}

/// Which form the built-ins select, and the currency named through `--currency`.
fn resolve_form(g: &Globals) -> Result<(FormKind, Option<&str>), UsageError> {
    let mut picked: Vec<FormKind> = Vec::new();
    if g.cardinal {
        picked.push(FormKind::Cardinal);
    }
    if g.ordinal {
        picked.push(FormKind::Ordinal);
    }
    if g.currency.is_some() {
        picked.push(FormKind::Currency);
    }
    if let Some(form) = &g.form {
        let kind: FormKind = form.parse().map_err(|_| {
            UsageError::with_hint(
                format!("Unknown form: {form}"),
                "Expected cardinal, ordinal or currency.",
            )
        })?;
        if !picked.contains(&kind) {
            picked.push(kind);
        }
    }
    match picked.as_slice() {
        [] => Ok((FormKind::Cardinal, None)),
        [one] => Ok((*one, g.currency.as_deref())),
        many => Err(UsageError::new(format!(
            "Conflicting forms: {}",
            many.iter().map(|k| k.name()).collect::<Vec<_>>().join(", ")
        ))),
    }
}

/// snake_case option name → kebab-case flag (`hundred_pairing` → `hundred-pairing`).
fn to_flag(name: &str) -> String {
    name.replace('_', "-")
}

/// Positional values plus `(option, value)` pairs from the language flags.
type LanguageArgs = (Vec<String>, Vec<(String, String)>);

/// Resolves the language flags and positional values of the second pass.
fn parse_language_args(
    rest: &[String],
    form: &Form,
    extra: &[String],
) -> Result<LanguageArgs, UsageError> {
    let mut values = Vec::new();
    let mut options: Vec<(String, String)> = Vec::new();
    let find = |flag: &str| form.options.iter().find(|o| to_flag(o.name) == flag);

    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        if !arg.starts_with("--") || looks_numeric(arg) {
            if arg.starts_with('-') && !looks_numeric(arg) {
                return Err(UsageError::with_hint(
                    format!("Unknown flag: {arg}"),
                    "Run n2words --help for the built-in flags.",
                ));
            }
            values.push(arg.to_string());
            i += 1;
            continue;
        }
        let (flag, inline) = match arg[2..].split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (&arg[2..], None),
        };
        if let Some(spec) = find(flag) {
            let value = match (matches!(spec.kind, OptionKind::Bool), inline) {
                (_, Some(v)) => v,
                (true, None) => "true".to_string(),
                (false, None) => {
                    i += 1;
                    rest.get(i)
                        .cloned()
                        .ok_or_else(|| UsageError::new(format!("Flag --{flag} needs a value")))?
                }
            };
            options.push((spec.name.to_string(), value));
        } else if let Some(spec) = flag
            .strip_prefix("no-")
            .and_then(find)
            .filter(|s| matches!(s.kind, OptionKind::Bool))
        {
            options.push((spec.name.to_string(), "false".to_string()));
        } else {
            let known: Vec<String> = form
                .options
                .iter()
                .map(|o| format!("--{}", to_flag(o.name)))
                .collect();
            let hint = if known.is_empty() {
                "This form takes no options.".to_string()
            } else {
                format!("Options for this form: {}", known.join(", "))
            };
            return Err(UsageError::with_hint(
                format!("Unknown flag: --{flag}"),
                hint,
            ));
        }
        i += 1;
    }

    // `--option key=value` reaches any option by its declared name.
    for item in extra {
        let (key, value) = item.split_once('=').ok_or_else(|| {
            UsageError::with_hint(
                format!("Malformed --option {item}"),
                "Expected --option key=value.",
            )
        })?;
        options.push((key.to_string(), value.to_string()));
    }
    Ok((values, options))
}

fn load_language(input: &str) -> Result<&'static Language, UsageError> {
    language(input).ok_or_else(|| {
        let needle = input.to_ascii_lowercase();
        let suggestions: Vec<&str> = languages()
            .iter()
            .map(|l| l.code)
            .filter(|c| {
                let c = c.to_ascii_lowercase();
                c.starts_with(&needle) || needle.starts_with(&c)
            })
            .take(5)
            .collect();
        let hint = if suggestions.is_empty() {
            "Run n2words --list to see every language code.".to_string()
        } else {
            format!("Did you mean: {}", suggestions.join(", "))
        };
        UsageError::with_hint(format!("Unknown language: {input}"), hint)
    })
}

// ============================================================================
// Rendering
// ============================================================================

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_options(options: &[(String, String)]) -> String {
    let fields: Vec<String> = options
        .iter()
        .map(|(k, v)| format!("{}:{}", json_string(k), json_string(v)))
        .collect();
    format!("{{{}}}", fields.join(","))
}

fn describe_kind(lang: &Language) -> String {
    match lang.kind {
        Kind::Full => "full implementation".to_string(),
        Kind::Profile { base } => format!("locale profile of {base}"),
        Kind::Alias { target } => format!("alias of {target}"),
    }
}

fn option_usage(spec: &OptionSpec) -> String {
    match spec.kind {
        OptionKind::Bool => format!("--{f} / --no-{f}", f = to_flag(spec.name)),
        OptionKind::Text => format!("--{} <text>", to_flag(spec.name)),
        OptionKind::Enum(_) => format!(
            "--{} <{}>",
            to_flag(spec.name),
            spec.values().unwrap_or_default().join("|")
        ),
        OptionKind::Currency(_) => format!("--{} <CODE>", to_flag(spec.name)),
    }
}

/// The description, with a currency option's accepted codes appended.
fn option_description(spec: &OptionSpec) -> String {
    match spec.kind {
        OptionKind::Currency(_) => {
            format!(
                "{}; one of {}",
                spec.description(),
                spec.values().unwrap_or_default().join(", ")
            )
        }
        _ => spec.description().to_string(),
    }
}

fn render_general_help() -> String {
    format!(
        "n2words {version}: spell numbers in words

Usage: n2words [values...] --lang <code> [--cardinal | --ordinal | --currency <CODE> | --form <form>] [options]
       n2words --list
       n2words --help --lang <code>

Values are read as text and passed through untouched, so precision is never
lost. With no values, one is read per line from stdin.

Built-in flags:
  -l, --lang <code>       Language: a bare tag (en, de) or a region/script code (en-GB, zh-Hans-CN)
  -f, --form <form>       cardinal (default), ordinal or currency
      --cardinal          Same as --form cardinal
      --ordinal           Same as --form ordinal
      --currency <CODE>   The currency form, naming the ISO 4217 currency
      --option key=value  Set any option of the selected language and form by name
  -j, --json              One JSON record per line: {{input, output, lang, form, options}} or {{input, error}}
      --list              Every entry point, its kind and the forms it supports
  -h, --help              This help, or a language's forms and options with --lang
  -v, --version           Print the version

Every flag past the built-ins is the selected language's own: `n2words --help --lang es-ES`
lists them. Booleans also accept --flag=false.

Exit codes: 0 success, 1 usage error, 2 at least one value could not be converted.
",
        version = n2words::VERSION
    )
}

fn render_language_help(lang: &Language) -> String {
    let mut out = format!(
        "{} — {} ({})\n\nForms:\n",
        lang.code,
        lang.name,
        describe_kind(lang)
    );
    for (kind, form) in lang.forms() {
        out.push_str(&format!(
            "  {:<10} up to {}\n",
            kind.name(),
            form.max.largest_display()
        ));
    }
    let with_options: Vec<_> = lang
        .forms()
        .filter(|(_, f)| !f.options.is_empty())
        .collect();
    if with_options.is_empty() {
        out.push_str("\nNo options.\n");
    } else {
        out.push_str("\nOptions:\n");
        for (kind, form) in with_options {
            out.push_str(&format!("  {}:\n", kind.name()));
            let usages: Vec<String> = form.options.iter().map(option_usage).collect();
            let width = usages.iter().map(String::len).max().unwrap_or(0);
            for (spec, usage) in form.options.iter().zip(usages) {
                let default = match spec.default_value() {
                    Some(d) => format!(" (default: {d})"),
                    None => " (required)".to_string(),
                };
                out.push_str(&format!(
                    "    {usage:<width$}  {}{default}\n",
                    option_description(spec)
                ));
            }
        }
    }
    out
}

fn render_list(json: bool) -> String {
    if json {
        let entries: Vec<String> = languages()
            .iter()
            .map(|lang| {
                let forms: Vec<String> = lang
                    .forms()
                    .map(|(k, f)| {
                        let max = match f.max.exponent() {
                            Some(e) => format!("\"10^{e} - 1\""),
                            None => "null".to_string(),
                        };
                        format!("{}:{{\"max\":{max}}}", json_string(k.name()))
                    })
                    .collect();
                let (kind, target) = match lang.kind {
                    Kind::Full => ("full", None),
                    Kind::Profile { base } => ("profile", Some(base)),
                    Kind::Alias { target } => ("alias", Some(target)),
                };
                format!(
                    "{{\"code\":{},\"name\":{},\"kind\":{},\"target\":{},\"forms\":{{{}}}}}",
                    json_string(lang.code),
                    json_string(lang.name),
                    json_string(kind),
                    target.map_or("null".to_string(), json_string),
                    forms.join(",")
                )
            })
            .collect();
        return format!("[{}]\n", entries.join(","));
    }
    let mut out = String::new();
    for lang in languages() {
        let forms: Vec<&str> = lang.forms().map(|(k, _)| k.name()).collect();
        let forms = if forms.is_empty() {
            "(no forms yet)".to_string()
        } else {
            forms.join(", ")
        };
        out.push_str(&format!(
            "{:<12} {:<34} {:<28} {forms}\n",
            lang.code,
            lang.name,
            describe_kind(lang)
        ));
    }
    out
}

// ============================================================================
// Main
// ============================================================================

fn run(args: &[String]) -> Result<u8, UsageError> {
    let g = parse_globals(args)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if g.version {
        writeln!(out, "n2words {}", n2words::VERSION).ok();
        return Ok(0);
    }
    if g.list {
        out.write_all(render_list(g.json).as_bytes()).ok();
        return Ok(0);
    }
    if g.help {
        match &g.lang {
            Some(code) => out
                .write_all(render_language_help(load_language(code)?).as_bytes())
                .ok(),
            None => out.write_all(render_general_help().as_bytes()).ok(),
        };
        return Ok(0);
    }

    let Some(code) = &g.lang else {
        return Err(UsageError::with_hint(
            "Missing --lang",
            "Pick a language: n2words 42 --lang en (see n2words --list).",
        ));
    };
    let lang = load_language(code)?;
    let (kind, currency) = resolve_form(&g)?;
    let form = lang.form(kind).ok_or_else(|| {
        let available: Vec<&str> = lang.forms().map(|(k, _)| k.name()).collect();
        UsageError::with_hint(
            format!("{} does not support the {kind} form", lang.code),
            format!("Available: {}", available.join(", ")),
        )
    })?;

    let (values, mut options) = parse_language_args(&g.rest, form, &g.options)?;
    if let Some(currency) = currency {
        // Named through the built-in, so it wins over a `--currency` flag or
        // `--option currency=...`, and is passed even to an alias that
        // deliberately has no default.
        options.retain(|(k, _)| k != "currency");
        options.push(("currency".to_string(), currency.to_string()));
    }
    let option_refs: Vec<(&str, &str)> = options
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let mut failed = false;
    let mut emit = |input: &str| {
        let result = form.convert(input, &option_refs);
        match (&result, g.json) {
            (Ok(output), false) => writeln!(out, "{output}").ok(),
            (Ok(output), true) => writeln!(
                out,
                "{{\"input\":{},\"output\":{},\"lang\":{},\"form\":{},\"options\":{}}}",
                json_string(input),
                json_string(output),
                json_string(lang.code),
                json_string(kind.name()),
                json_options(&options)
            )
            .ok(),
            (Err(err), false) => {
                failed = true;
                eprintln!("n2words: {input}: {err}");
                Some(())
            }
            (Err(err), true) => {
                failed = true;
                writeln!(
                    out,
                    "{{\"input\":{},\"error\":{}}}",
                    json_string(input),
                    json_string(&err.to_string())
                )
                .ok()
            }
        };
        // A malformed option is the same for every value: stop after the first.
        matches!(&result, Err(err) if err.is_malformed_argument() || matches!(err, Error::InvalidOptionValue { .. }))
    };

    if values.is_empty() {
        if io::stdin().is_terminal() {
            return Err(UsageError::with_hint(
                "No value to convert",
                "Pass values as arguments or pipe them in, one per line.",
            ));
        }
        for line in io::stdin().lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if emit(line) {
                break;
            }
        }
    } else {
        for value in &values {
            if emit(value) {
                break;
            }
        }
    }

    Ok(if failed { EXIT_CONVERSION } else { 0 })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => ExitCode::from(code),
        Err(UsageError { message, hint }) => {
            eprintln!("n2words: {message}");
            if let Some(hint) = hint {
                eprintln!("{hint}");
            }
            ExitCode::from(EXIT_USAGE)
        }
    }
}
