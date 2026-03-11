# ah-ah-ah

## Project Layout

This is a single Rust library crate. Source lives in `src/`.

## Commands

Use `just` for all dev tasks:

```
just check          # fmt + clippy + deny + test + doc-test (run before pushing)
just test           # cargo nextest run
just clippy         # lint with pinned toolchain
just fmt            # cargo fmt --all
just deny           # security/license audit
just fix            # auto-fix clippy warnings
just bench          # run benchmarks
just release-check  # pre-release validation
just outdated       # check for outdated dependencies
just upgrade        # update deps in Cargo.toml and Cargo.lock
```

**Tests use `cargo nextest run`**, not `cargo test`. Doc tests are separate: `cargo test --doc`.

## Rust Conventions

- **Edition 2024**, MSRV **1.89.0**, toolchain pinned in `rust-toolchain.toml`
- `unsafe_code = "deny"` — no unsafe unless explicitly allowed with a `// SAFETY:` comment
- Clippy `all` = warn, `nursery` = warn — treat warnings as errors in CI
- Use `thiserror` for error types

## Do Not

- Commit anything in `target/`
- Add dependencies without checking `deny.toml` license policy (`just deny`)
- Skip `--all-targets --all-features` when running clippy
- Use `cargo test` instead of `cargo nextest run`
- Run raw cargo commands when a `just` recipe exists
