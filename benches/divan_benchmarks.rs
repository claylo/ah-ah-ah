//! Divan benchmarks for ah-ah-ah
//!
//! Wall-clock time benchmarks for token counting backends.
//! Run with: `cargo bench --bench divan_benchmarks`

#![allow(missing_docs)]

use ah_ah_ah::{count_tokens, Backend, MarkdownDecomposer};

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Corpus constants
// ---------------------------------------------------------------------------

const SHORT_TEXT: &str = "Hello, world!";

const ENGLISH_PROSE: &str = "\
The quick brown fox jumps over the lazy dog. This is a moderately long passage \
of English text that exercises common token boundaries including punctuation, \
contractions like don't and won't, hyphenated words like state-of-the-art, and \
numbers like 42 and 3.14159. It also includes some longer words like \
internationalization, pneumonoultramicroscopicsilicovolcanoconiosis, and \
antidisestablishmentarianism to test multi-byte token matching.";

const MARKDOWN_TABLE: &str = "\
| Name       | Age | City          | Score |
|------------|-----|---------------|-------|
| Alice      | 30  | New York      | 95.5  |
| Bob        | 25  | San Francisco | 87.3  |
| Charlie    | 35  | London        | 92.1  |
| Diana      | 28  | Tokyo         | 88.7  |
| Eve        | 31  | Berlin        | 91.0  |
";

const RUST_CODE: &str = r#"
fn process(items: &[Item]) -> Result<Vec<Output>, Error> {
    items
        .iter()
        .filter(|item| item.is_valid() || item.is_pending())
        .map(|item| match item.kind {
            Kind::Alpha => transform_alpha(item),
            Kind::Beta | Kind::Gamma => transform_beta_gamma(item),
            _ => Err(Error::UnsupportedKind(item.kind)),
        })
        .collect()
}
"#;

const MIXED_MARKDOWN: &str = "\
# Performance Report

Some introductory prose about the benchmark results.

| Backend | Ops/sec | Latency (ms) | Memory (MB) |
|---------|---------|--------------|-------------|
| Claude  | 150,000 | 0.007        | 12.4        |
| OpenAI  | 85,000  | 0.012        | 8.1         |

The results show that the Claude backend is significantly faster due to
the Aho-Corasick automaton's linear-time matching, while OpenAI's BPE
encoding trades throughput for exact token counts.

## Methodology

All benchmarks were run on a single core with warm caches. The p99
latency was measured over 10,000 iterations with | pipe characters |
in the surrounding text to test the fast-path heuristic.
";

// Build a ~10KB text for throughput measurement.
fn large_text() -> String {
    ENGLISH_PROSE.repeat(25)
}

// ---------------------------------------------------------------------------
// Claude backend
// ---------------------------------------------------------------------------

#[divan::bench]
fn claude_short(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(SHORT_TEXT, None, Backend::Claude, None));
}

#[divan::bench]
fn claude_prose(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(ENGLISH_PROSE, None, Backend::Claude, None));
}

#[divan::bench]
fn claude_code(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(RUST_CODE, None, Backend::Claude, None));
}

#[divan::bench]
fn claude_large(bencher: divan::Bencher) {
    let text = large_text();
    bencher.bench(|| count_tokens(&text, None, Backend::Claude, None));
}

// ---------------------------------------------------------------------------
// OpenAI backend
// ---------------------------------------------------------------------------

#[divan::bench]
fn openai_short(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(SHORT_TEXT, None, Backend::Openai, None));
}

#[divan::bench]
fn openai_prose(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(ENGLISH_PROSE, None, Backend::Openai, None));
}

#[divan::bench]
fn openai_code(bencher: divan::Bencher) {
    bencher.bench(|| count_tokens(RUST_CODE, None, Backend::Openai, None));
}

#[divan::bench]
fn openai_large(bencher: divan::Bencher) {
    let text = large_text();
    bencher.bench(|| count_tokens(&text, None, Backend::Openai, None));
}

// ---------------------------------------------------------------------------
// Decomposition
// ---------------------------------------------------------------------------

#[divan::bench]
fn decompose_table_claude(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(MARKDOWN_TABLE, None, Backend::Claude, Some(&md)));
}

#[divan::bench]
fn decompose_table_openai(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(MARKDOWN_TABLE, None, Backend::Openai, Some(&md)));
}

#[divan::bench]
fn decompose_mixed_claude(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(MIXED_MARKDOWN, None, Backend::Claude, Some(&md)));
}

#[divan::bench]
fn decompose_mixed_openai(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(MIXED_MARKDOWN, None, Backend::Openai, Some(&md)));
}

// ---------------------------------------------------------------------------
// Fast-path heuristic: code with pipes should NOT trigger the parser
// ---------------------------------------------------------------------------

#[divan::bench]
fn fastpath_code_with_pipes(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(RUST_CODE, None, Backend::Claude, Some(&md)));
}

#[divan::bench]
fn fastpath_prose_no_pipes(bencher: divan::Bencher) {
    let md = MarkdownDecomposer;
    bencher.bench(|| count_tokens(ENGLISH_PROSE, None, Backend::Claude, Some(&md)));
}
