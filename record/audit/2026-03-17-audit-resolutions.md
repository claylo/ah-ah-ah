# Resolutions for 2026-03-17 Security and Code-Quality Audit

Date: 2026-03-17

## Finding 1 (Medium): RESOLVED — Table separator heuristic missed pipe-less tables

`has_table_separator()` in `src/decompose.rs` required `starts_with('|')`, rejecting valid separator rows like `----|-----`. Removed that requirement; now any line composed entirely of `-:| \t` with at least one dash and one pipe qualifies.

Tests added:
- `table_without_leading_pipe_detected`
- `table_without_outer_pipes_mixed_with_prose`
- `separator_with_alignment_no_leading_pipe`

## Finding 2 (Medium): RESOLVED — Exact backends now skip decomposition

Added `Backend::is_exact()` (`true` for OpenAI, `false` for Claude). `count_tokens()` skips the decomposer when the backend is exact, emitting `tracing::debug!` when doing so. This preserves the OpenAI "exact o200k_base" contract regardless of what the caller passes.

Tests added:
- `openai_ignores_decomposer` — asserts raw == decomposed counts for OpenAI
- `openai_exact_flag` — locks down `is_exact()` return values

Finding 2's fix also neutralizes Finding 3's impact on OpenAI, since decomposition no longer runs for exact backends.

## Finding 3 (Low): OPEN

Table counting still over-splits on escaped/inline-code pipes. Conservative for Claude (safe direction), and now irrelevant for OpenAI per Finding 2's fix.

## Validation

68 tests pass, clippy clean, 2 doc-tests pass.
