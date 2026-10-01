# AGENTS.md

`overseer-launch` is a Rust CLI, binary name `overseer`, that execs an
overseer session from a named profile. See `README.md` for use. The
skill it serves is [overseer](https://github.com/photuris/overseer);
`profiles.example.toml` here and `resources/profiles.example.toml`
there are twins and change together.

## Conventions

- Crate `overseer-launch`, binary `overseer`. Edition 2024, Rust 1.98.
- Dependencies: `clap`, `serde`, `serde_json`, `toml`. No dev
  dependencies. Do not add one without a reason in the commit message.
- `max_width = 79` in `rustfmt.toml`. No `unwrap`, `expect`, or
  `panic!` outside tests.
- Every item gets a doc comment, private items too; tests are the
  exception, their `should_…` names document them. `# Errors` on every
  public function that returns `Result`.
- A blank line before every control statement, `return`, and tail
  expression, unless it is first in its block.
- Tests: `should_<expected>_when_<state>`, unit tests in
  `mod tests { mod <unit> }` beside the code, CLI tests in
  `tests/cli.rs` run the built binary with `--print` or `init` against
  `tests/fixtures/profiles.toml`. Never launch a paid agent in a test;
  the one real exec test uses `sh`.
- Before each commit: `cargo fmt --check`,
  `cargo clippy --all-targets --all-features --locked -- -D warnings`,
  `cargo test --locked`.
- Commit messages: lower case, imperative, one line, no emoji, no
  attribution lines.

## Design decisions

- No profile auto-detection from the running harness. Several
  profiles may share an overseer command with different implementers,
  so the name is always explicit or the file's `default`.
- A launch owns the roster variables: it removes inherited
  `OVERSEER_*` and `HERDR_OVERSEER` before adding the profile's own.
  Otherwise a profile without a judge would inherit one from the
  shell that launched the previous session.
- `init` never overwrites. Exit 1 with the path.
- No `list`, `add`, or `edit`. The TOML file is the interface; an
  unknown name lists the known ones in its error.
