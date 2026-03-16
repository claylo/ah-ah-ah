# Handoff: Initial token counting library

**Date:** 2026-03-11
**Branch:** main
**State:** Green

> Green = tests pass, safe to continue. Yellow = tests pass but known issues exist. Red = broken state, read Landmines first.

## Where things stand

ah-ah-ah is a working single-crate token counting library with two backends (Claude greedy longest-match, OpenAI o200k_base BPE) and a `Decomposer` trait for boundary-aware counting of structured content. 17 tests pass, clippy clean, `just check` fully green.

## Decisions made

- **Flat crate, not workspace** — single library, no `crates/` directory. `src/` at root.
- **No schemars** — too heavy for this crate. Consumers add their own JSON schema derives.
- **o200k_base, not cl100k** — GPT-5/gpt-oss use o200k. The `Backend::Openai` variant uses `bpe_openai::o200k_base()`.
- **No clap** — pure library, no CLI concerns.
- **`Decomposer` trait** — `fn count(&self, text, raw_count)` pattern. `MarkdownDecomposer` ships as the included implementation using pulldown-cmark for table boundaries.
- **No probe feature** — token verification (ctoc-style HTTP probing) is a separate concern, possibly a standalone CLI later.

## What's next

1. **`MarkdownDecomposer` fast-path heuristic** — the current `contains('|')` check triggers pulldown-cmark parsing on source code with pipes (match arms, bitwise OR, closures). Add a cheap separator-row scan (`|---` or `|:--`) before invoking the parser. Callers who know they have code should also just pass `None` for the decomposer. Do both.
2. **Benchmarks** — add divan benchmarks comparing against other token counting crates (tiktoken-rs, etc.). Wire up `benches/benchmarks.kdl` and `benches/divan_benchmarks.rs`.
3. **More tests** — edge cases: unicode, large inputs, boundary conditions, multi-table documents, nested structures, empty inputs for each code path, decomposer trait contract tests. Include source-code-with-pipes tests for the heuristic fix.
4. **bito-lint integration** — once stable, make ah-ah-ah a dependency of bito-lint-core, replacing its inline `tokens.rs`. Not yet — get benchmarks and tests solid first.

## Landmines

- **`claude_vocab.json` is 475KB embedded via `include_str!`** — the Aho-Corasick automaton is built lazily on first use (`LazyLock`). First call to Claude backend has initialization cost.
