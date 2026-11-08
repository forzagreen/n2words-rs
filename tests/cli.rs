//! End-to-end tests of the `n2words` binary: output, exit codes, JSON and
//! stdin streaming.

use std::io::Write;
use std::process::{Command, Stdio};

struct Run {
    stdout: String,
    stderr: String,
    code: i32,
}

fn n2words(args: &[&str], stdin: Option<&str>) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_n2words"));
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = cmd.spawn().expect("binary runs");
    if let Some(input) = stdin {
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(input.as_bytes())
            .expect("stdin written");
    }
    let output = child.wait_with_output().expect("binary exits");
    Run {
        stdout: String::from_utf8(output.stdout).expect("utf-8 stdout"),
        stderr: String::from_utf8(output.stderr).expect("utf-8 stderr"),
        code: output.status.code().expect("exit code"),
    }
}

#[test]
fn converts_positional_values() {
    let run = n2words(&["42", "-l", "en"], None);
    assert_eq!((run.code, run.stdout.as_str()), (0, "forty-two\n"));

    let run = n2words(&["1", "-17.42", "--lang", "en-US"], None);
    assert_eq!(
        (run.code, run.stdout.as_str()),
        (0, "one\nminus seventeen point forty-two\n")
    );
}

#[test]
fn forms_and_currency() {
    assert_eq!(
        n2words(&["42", "-l", "en-GB", "--ordinal"], None).stdout,
        "forty-second\n"
    );
    assert_eq!(
        n2words(&["42", "-l", "en-GB", "-f", "ordinal"], None).stdout,
        "forty-second\n"
    );
    assert_eq!(
        n2words(&["42.50", "-l", "en-US", "--currency", "GBP"], None).stdout,
        "forty-two pounds and fifty pence\n"
    );
    assert_eq!(
        n2words(&["42.50", "-l", "en-US", "--form", "currency"], None).stdout,
        "forty-two dollars and fifty cents\n"
    );
    // A bare tag has no default currency; `--currency` names one.
    assert_eq!(
        n2words(&["1", "-l", "en", "--currency", "EUR"], None).stdout,
        "one euro\n"
    );
    let run = n2words(&["1", "-l", "en", "--form", "currency"], None);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("required"), "{}", run.stderr);
}

#[test]
fn language_options_become_flags() {
    assert_eq!(
        n2words(&["101", "-l", "en-US", "--and"], None).stdout,
        "one hundred and one\n"
    );
    assert_eq!(
        n2words(&["1501", "-l", "en-US", "--and", "--hundred-pairing"], None).stdout,
        "fifteen hundred and one\n"
    );
    assert_eq!(
        n2words(
            &[
                "1501",
                "-l",
                "en-US",
                "--and=true",
                "--hundred-pairing=false"
            ],
            None
        )
        .stdout,
        "one thousand five hundred and one\n"
    );
    assert_eq!(
        n2words(
            &["1.50", "-l", "en-US", "--form", "currency", "--no-and"],
            None
        )
        .stdout,
        "one dollar fifty cents\n"
    );
    assert_eq!(
        n2words(&["101", "-l", "en-US", "--option", "and=true"], None).stdout,
        "one hundred and one\n"
    );

    let run = n2words(&["42", "-l", "en-US", "--bogus"], None);
    assert_eq!(run.code, 1);
    assert!(
        run.stderr.contains("Unknown flag: --bogus"),
        "{}",
        run.stderr
    );

    let run = n2words(&["42", "-l", "en-US", "--and=maybe"], None);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("must be one of"), "{}", run.stderr);
}

#[test]
fn usage_errors_exit_1() {
    let run = n2words(&["42"], None);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("Missing --lang"), "{}", run.stderr);

    let run = n2words(&["42", "-l", "xx-YY"], None);
    assert_eq!(run.code, 1);
    assert!(
        run.stderr.contains("Unknown language: xx-YY"),
        "{}",
        run.stderr
    );

    let run = n2words(&["42", "-l", "en-U"], None);
    assert_eq!(run.code, 1);
    assert!(
        run.stderr.contains("Did you mean:") && run.stderr.contains("en-US"),
        "{}",
        run.stderr
    );

    let run = n2words(&["42", "-l", "en", "--cardinal", "--ordinal"], None);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("Conflicting forms"), "{}", run.stderr);

    let run = n2words(&["42", "-l", "en", "-f", "plural"], None);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("Unknown form"), "{}", run.stderr);
}

#[test]
fn conversion_errors_exit_2_but_continue() {
    let run = n2words(&["1", "1e100", "2", "-l", "en"], None);
    assert_eq!(run.code, 2);
    assert_eq!(run.stdout, "one\ntwo\n");
    assert!(run.stderr.contains("1e100"), "{}", run.stderr);
}

#[test]
fn json_output() {
    let run = n2words(&["42", "-l", "en-GB", "--ordinal", "--json"], None);
    assert_eq!(
        run.stdout,
        "{\"input\":\"42\",\"output\":\"forty-second\",\"lang\":\"en-GB\",\"form\":\"ordinal\",\"options\":{}}\n"
    );

    let run = n2words(&["0", "-l", "en", "--ordinal", "-j"], None);
    assert_eq!(run.code, 2);
    assert!(
        run.stdout.starts_with("{\"input\":\"0\",\"error\":"),
        "{}",
        run.stdout
    );
    assert!(run.stderr.is_empty());

    let run = n2words(&["1.5", "-l", "en-US", "--currency", "EUR", "--json"], None);
    assert!(
        run.stdout.contains("\"options\":{\"currency\":\"EUR\"}"),
        "{}",
        run.stdout
    );
}

#[test]
fn reads_stdin_one_value_per_line() {
    let run = n2words(&["-l", "en"], Some("1\n\n2\n 3 \n"));
    assert_eq!((run.code, run.stdout.as_str()), (0, "one\ntwo\nthree\n"));

    let run = n2words(&["-l", "en", "--json"], Some("1\nx\n"));
    assert_eq!(run.code, 2);
    assert_eq!(run.stdout.lines().count(), 2);
}

#[test]
fn list_help_and_version() {
    let run = n2words(&["--list"], None);
    assert_eq!(run.code, 0);
    assert!(run.stdout.contains("en-US"), "{}", run.stdout);
    assert!(run.stdout.contains("alias of en-US"), "{}", run.stdout);
    assert!(
        run.stdout.contains("locale profile of en-GB"),
        "{}",
        run.stdout
    );

    let run = n2words(&["--list", "--json"], None);
    assert!(run.stdout.starts_with("[{\"code\":"), "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "\"code\":\"en-US\",\"name\":\"American English\",\"kind\":\"full\",\"target\":null"
        ),
        "{}",
        run.stdout
    );

    let run = n2words(&["--help"], None);
    assert_eq!(run.code, 0);
    assert!(run.stdout.contains("Usage: n2words"), "{}", run.stdout);

    let run = n2words(&["--help", "--lang", "en-US"], None);
    assert!(
        run.stdout
            .contains("--hundred-pairing / --no-hundred-pairing"),
        "{}",
        run.stdout
    );
    assert!(run.stdout.contains("10^66 - 1"), "{}", run.stdout);

    let run = n2words(&["--version"], None);
    assert_eq!(
        run.stdout,
        format!("n2words {}\n", env!("CARGO_PKG_VERSION"))
    );
}
