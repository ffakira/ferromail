# Contributing

Thanks for taking an interest. ferromail is small and pre-1.0, so the bar is
mostly about keeping one property intact: a string that came from outside the
program must not be able to become markup. Read
[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) before changing anything in `markup`.

## Using AI

AI assistance is allowed and does not need to be justified.

Credit it in the commit trailer, on the commits where it was used:

```
Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
```

Name the model, not just the vendor, so the history says what actually wrote
the change.

Two expectations that apply to AI-assisted work in particular:

- You are responsible for the diff. Review it as if you had typed it. An
  explanation that reads well is not evidence that the code is right.
- Claims in comments and docs must be checked against the code. This repository
  has already had documentation confidently describe behaviour that did not
  exist. If a comment says a client does something, or a type prevents
  something, verify it rather than repeating it.

## API stability

Breaking changes are fine right now. While the version is `0.0.x` they ship in
patch releases, marked **Breaking** in [CHANGELOG.md](CHANGELOG.md).

**From 0.1.0 that stops.** Breaking changes will need a minor version bump and
a reason that outweighs the churn. Preferences, in order:

1. Add rather than change. Public enums are `#[non_exhaustive]` precisely so a
   new `Tag`, `AttrName` or `Node` variant is additive.
2. Deprecate before removing. `#[deprecated(since = "...", note = "use X")]`
   for at least one minor release.
3. If it must break, say so in the commit subject with `!`
   (`feat(markup)!: ...`), and add a **Breaking** entry to the changelog
   explaining what a caller has to do.

So if an API decision feels wrong, now is the time to raise it.

## Getting set up

```sh
cargo test                                                 # unit, property, doctests
docker compose up -d                                       # local SMTP catcher
cargo run --example send                                   # preview at localhost:8025
cargo test --test client_support -- --ignored --nocapture  # client-support report
```

The client-support test is `#[ignore]`d because it needs the container. It is
not run in CI.

## What CI checks

Every push to `main` or `dev`, and every pull request:

| job | what |
|---|---|
| `test` | `cargo test --all-features --locked` on Linux, macOS and Windows |
| `lint` | `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings` |
| `msrv 1.85` | `cargo check --lib` on Rust 1.85.0 |

`RUSTFLAGS: -D warnings`, so a warning fails the build.

Two things worth knowing before CI surprises you:

- The toolchain is `@stable` and floats. A new clippy lint can turn the build
  red with no change from you. That is deliberate: the lints have been worth
  fixing. Run `rustup update` if your local clippy is behind.
- The MSRV job checks `--lib` only, not `--all-targets`. `rust-version` is a
  promise to consumers, and a consumer compiles the library, which has no
  runtime dependencies. Dev-dependencies need a newer Rust, and pulling them
  into that job would fail for a reason no consumer can hit.

## Commits and branches

Work happens on `dev` and reaches `main` by pull request. Tags on `main` drive
the release workflow.

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
`docs:`, `test:`, `ci:`, `chore:`, `style:`, with an optional scope
(`feat(button):`) and `!` for a breaking change.

Write the body for someone reading `git log` in a year with no memory of the
discussion. Say why, not what: the diff already says what.

## House rules

**No runtime dependencies.** ferromail has none, and adding one needs a better
reason than convenience. Dev-dependencies are fine.

**Validation lives in a type, not in a function that promises to be careful.**
If you find yourself writing a check at a call site, the check probably belongs
in a constructor that is the only way to build the value.

**Adding a tag or attribute is two edits.** A variant in `markup::Tag` or
`markup::AttrName`, and an arm in the matching table in `src/macros.rs`. Miss
the second and `html!` reports the name as unknown.

**Reject more than the spec allows.** `StyleValue` refuses CSS functions
outright because an email client that silently renders `calc()` wrong is worse
than a compile error. Widening a validator needs a case, not a preference.

**Prose style.** No em dashes anywhere, including comments and commit messages.

## Tests

New behaviour needs a test that fails without the change. For anything in
`markup` or `render`, prefer asserting on rendered output rather than internal
state, since the output is the contract.

If you fix something a property test could have caught, consider extending the
strategy in `tests/render_props.rs` rather than only adding a unit test. The
saved cases in `tests/render_props.proptest-regressions` are checked in
deliberately: they replay on every run, so commit the file when it changes.

## Licence

By contributing you agree your work is dual-licensed under
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE), matching the crate.
