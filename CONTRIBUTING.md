# Contributing to n2words

## Quick start

```bash
git clone https://github.com/forzagreen/n2words-rs.git
cd n2words-rs
cargo test
```

**Requirements:** Rust 1.85 or newer (`rustup update stable`), Git.

Useful commands:

```bash
cargo test                                     # everything: unit tests, fixtures, gates, CLI, doctests
cargo test --test conversions -- de-DE         # one language's fixtures (substring filter)
cargo test --test contracts                    # the behavioural gates
cargo clippy --all-targets -- -D warnings      # lint, as CI runs it
cargo fmt                                      # format
cargo run --example generate_languages_md      # regenerate LANGUAGES.md after changing a language
cargo run -- 42 -l en                          # the CLI from source
```

## Pull requests

1. Create a feature branch: `git checkout -b feat/add-xx-XX`
2. Make changes, then run `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test`
3. Regenerate `LANGUAGES.md` if you touched a language or an option
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/):
   `git commit -m "feat(ko-KR): add Korean"` or `fix(en-US): correct thousand handling`
5. Push and open a PR against `main`; its title must be a conventional commit too

Scopes are BCP 47 codes (`en-US`, a bare subtag such as `en` for a family, or
a comma-separated list) or project areas (`core`, `cli`, `docs`, `ci`, `release`, `deps`).

## Adding or fixing a language

Every language lives in `src/lang/{code_in_snake_case}.rs` and is listed in
`src/lang/mod.rs`. [docs/porting.md](docs/porting.md) explains the three kinds
of module (full implementation, locale profile, bare-tag alias), the helpers
the core provides, the `options!` macro and the checklist a language must
clear. Test cases go in `tests/fixtures/{code}.json`:

```json
{ "code": "ko-KR", "cardinal": [ { "input": 42, "expected": "사십이" } ], "ordinal": [...], "currency": [...] }
```

Inputs are JSON numbers, strings, or `{ "bigint": "123..." }`; options are an
object of option name to value. Each form needs at least five cases. The
generic gates in `tests/contracts.rs` then verify the declared ceiling, the
options contract and the currency vocabulary automatically; a new language
needs no other edit.

New languages are first added to the
[JavaScript library](https://github.com/forzagreen/n2words), whose fixtures
this port imports (`scripts/import-fixtures.mjs`), so that both stay in sync.
Corrections from native speakers are welcome in either repository.

## Releasing

Releases are cut by tag. Maintainers use [`cargo release`](https://github.com/crate-ci/cargo-release)
with the configuration in `release.toml`: it bumps the version, regenerates
`CHANGELOG.md` with [git-cliff](https://git-cliff.org/), commits, tags `vX.Y.Z`
and pushes. The `Release` workflow then publishes the crate to crates.io
(trusted publishing), creates the GitHub release with the changelog section
and attaches prebuilt `n2words` binaries.

```bash
cargo install cargo-release git-cliff
cargo release patch --execute   # or minor / major
```
