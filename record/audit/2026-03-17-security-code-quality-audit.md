# Security and Code-Quality Audit

Date: 2026-03-17

## Summary

- No critical or high-severity security issues were identified in the crate itself.
- `just check` passed cleanly, including `cargo fmt`, `clippy`, `cargo deny check`, `cargo nextest run`, and `cargo test --doc`.
- The main residual risk is correctness drift in markdown-aware counting, especially where the crate promises conservative Claude counts and exact OpenAI counts.

## Scope

- Reviewed repository metadata and security posture: [`Cargo.toml`](../../Cargo.toml), [`deny.toml`](../../deny.toml), [`SECURITY.md`](../../SECURITY.md), and [`.justfile`](../../.justfile).
- Reviewed the runtime code paths in [`src/claude.rs`](../../src/claude.rs), [`src/openai.rs`](../../src/openai.rs), [`src/decompose.rs`](../../src/decompose.rs), [`src/tokens.rs`](../../src/tokens.rs), and [`src/error.rs`](../../src/error.rs).
- Reviewed the test suite in [`tests/token_counting.rs`](../../tests/token_counting.rs) and the example/script entry points in [`examples/count.rs`](../../examples/count.rs) and [`scripts/gen-token-fixtures.sh`](../../scripts/gen-token-fixtures.sh).

## Findings

### 1. Medium: `MarkdownDecomposer` misses valid markdown tables without a leading `|`

Evidence:
[`src/decompose.rs:78-97`](../../src/decompose.rs) only treats a line as a table separator if the trimmed line starts with `|`.
[`src/decompose.rs:123-127`](../../src/decompose.rs) returns the raw tokenizer path when that heuristic fails.
This is narrower than `pulldown-cmark` table support.
Its own upstream test suite accepts forms like `Test|Table` / `----|-----` as tables.

Impact:
Valid markdown tables can silently bypass decomposition and fall back to raw greedy counting.
That matters most for the Claude backend, where the decomposer exists specifically to prevent cross-cell matches and undercounting.

Recommendation:
Relax the fast-path heuristic so it also recognizes separator rows without leading pipes, or skip the heuristic and trust `pulldown-cmark` once a plausible pipe-table candidate is present.
Add regression tests for header/separator forms without outer pipes.

### 2. Medium: decomposers can be applied to the OpenAI backend, breaking the "exact" count contract

Evidence:
[`src/tokens.rs:41-46`](../../src/tokens.rs) selects a backend-specific raw counter and then applies any provided decomposer without checking backend semantics.
[`src/decompose.rs:103-110`](../../src/decompose.rs) changes tokenization by splitting source on raw `|` boundaries and counting each pipe as one token.
The README advertises the OpenAI backend as exact `o200k_base` BPE.
The current test [`tests/token_counting.rs:524-533`](../../tests/token_counting.rs) only asserts that decomposed OpenAI counting returns a positive number, not that it remains exact.

Impact:
Callers can reasonably assume `Backend::Openai` is exact in all modes, but `Some(&MarkdownDecomposer)` invalidates that guarantee.
For budget enforcement, this can produce misleading counts and inconsistent behavior across backends.

Recommendation:
Either reject decomposers for exact backends, document the loss of exactness explicitly, or encode backend capabilities so decomposition is only available where conservative approximation is acceptable.
Add tests that lock down the intended behavior.

### 3. Low: table counting over-splits escaped or inline-code pipes inside cells

Evidence:
[`src/decompose.rs:105-109`](../../src/decompose.rs) splits every table line on every raw `|` byte.
That ignores markdown escaping and inline formatting.
`pulldown-cmark` accepts table cells containing escaped pipes and code spans with pipes as single cells, so the current implementation is not syntax-aware enough for those cases.

Impact:
Tables containing `\|` or inline code like `` `|` `` are over-segmented.
For the Claude backend this is conservative but noisy.
For the OpenAI backend it compounds finding 2 by drifting farther away from the exact tokenizer.

Recommendation:
Derive cell boundaries from parser events or parsed offsets rather than `split('|')`.
At minimum, add regression tests covering escaped pipes and inline code within table cells so the current tradeoff is explicit.

## Additional Notes

- [`src/error.rs:5-14`](../../src/error.rs) exports a recoverable `Error`/`Result` API, but [`src/claude.rs:17-23`](../../src/claude.rs) still uses `expect(...)` during lazy initialization.
  That is a low-probability reliability issue rather than a practical security bug, because the vocabulary is embedded at compile time.
  The API surface would be clearer if initialization either returned `Result` or the unused public error type were removed.

## Validation

Commands run:

- `just check`
- `rg --files`
- `rg -n "expect\\(|unwrap\\(|panic!|todo!|unimplemented!|unsafe|LazyLock|include_str!|from_str|split\\('|\\'\\)|find_iter|count_table|has_table_separator" src tests examples scripts`

Observed results:

- `cargo fmt --all`: passed
- `cargo +1.94.0 clippy --all-targets --all-features -- -D warnings`: passed
- `cargo deny check`: passed, with advisories/bans/licenses/sources all clean
- `cargo nextest run`: 64 tests passed
- `cargo test --doc`: 2 doc tests passed
