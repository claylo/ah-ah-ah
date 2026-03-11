#![allow(missing_docs)]

use ah_ah_ah::{Backend, Decomposer, MarkdownDecomposer, count_tokens};

#[test]
fn claude_backend_counts_tokens() {
    let report = count_tokens("Hello, world!", None, Backend::Claude, None);
    assert!(report.count > 0);
    assert_eq!(report.tokenizer, "claude");
}

#[test]
fn openai_backend_counts_tokens() {
    let report = count_tokens("Hello, world!", None, Backend::Openai, None);
    assert!(report.count > 0);
    assert_eq!(report.tokenizer, "openai");
}

#[test]
fn claude_overcounts_vs_openai() {
    let text = "The quick brown fox jumps over the lazy dog. \
                This is a longer passage of English text that should \
                demonstrate the conservative overcounting behavior of \
                the Claude tokenizer backend compared to OpenAI's exact \
                o200k_base encoding.";
    let claude = count_tokens(text, None, Backend::Claude, None);
    let openai = count_tokens(text, None, Backend::Openai, None);
    assert!(
        claude.count >= openai.count,
        "Claude ({}) should overcount vs OpenAI ({})",
        claude.count,
        openai.count
    );
}

#[test]
fn backend_default_is_claude() {
    assert_eq!(Backend::default(), Backend::Claude);
}

#[test]
fn backend_display_and_as_str() {
    assert_eq!(Backend::Claude.as_str(), "claude");
    assert_eq!(Backend::Openai.as_str(), "openai");
    assert_eq!(format!("{}", Backend::Claude), "claude");
    assert_eq!(format!("{}", Backend::Openai), "openai");
}

#[test]
fn detects_over_budget() {
    let report = count_tokens(
        "Hello, world! This is a test.",
        Some(1),
        Backend::default(),
        None,
    );
    assert!(report.over_budget);
    assert_eq!(report.budget, Some(1));
}

#[test]
fn within_budget() {
    let report = count_tokens("Hi", Some(100), Backend::default(), None);
    assert!(!report.over_budget);
}

#[test]
fn empty_text_returns_zero() {
    let report = count_tokens("", None, Backend::Claude, None);
    assert_eq!(report.count, 0);
    let report = count_tokens("", None, Backend::Openai, None);
    assert_eq!(report.count, 0);
}

#[test]
fn backend_serde_roundtrip() {
    let json = serde_json::to_string(&Backend::Claude).unwrap();
    assert_eq!(json, "\"claude\"");
    let back: Backend = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Backend::Claude);

    let json = serde_json::to_string(&Backend::Openai).unwrap();
    assert_eq!(json, "\"openai\"");
    let back: Backend = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Backend::Openai);
}

// ---------------------------------------------------------------------------
// Decomposer tests
// ---------------------------------------------------------------------------

#[test]
fn markdown_decomposer_table_counts_at_least_raw() {
    let table = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |\n";
    let md = MarkdownDecomposer;
    let raw = count_tokens(table, None, Backend::Claude, None);
    let aware = count_tokens(table, None, Backend::Claude, Some(&md));
    assert!(
        aware.count >= raw.count,
        "table-aware ({}) should be >= raw ({})",
        aware.count,
        raw.count
    );
}

#[test]
fn no_table_matches_raw() {
    let text = "The quick brown fox jumps over the lazy dog.";
    let md = MarkdownDecomposer;
    let raw = count_tokens(text, None, Backend::Claude, None);
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert_eq!(aware.count, raw.count);
}

#[test]
fn pipe_in_non_table_unchanged() {
    let text = "Use the || operator for logical OR.";
    let md = MarkdownDecomposer;
    // No markdown table structure, so raw path is used.
    let raw = count_tokens(text, None, Backend::Claude, None);
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert_eq!(aware.count, raw.count);
}

#[test]
fn mixed_table_and_prose() {
    let text = "Some prose before the table.\n\n\
                | Col A | Col B |\n\
                |-------|-------|\n\
                | x     | y     |\n\n\
                Some prose after the table.";
    let md = MarkdownDecomposer;
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert!(aware.count > 0, "should produce a positive count");
    let raw = count_tokens(text, None, Backend::Claude, None);
    assert!(
        aware.count >= raw.count,
        "table-aware ({}) should be >= raw ({})",
        aware.count,
        raw.count
    );
}

#[test]
fn empty_table_cells() {
    let table = "| | |\n|---|---|\n| | |\n";
    let md = MarkdownDecomposer;
    let count = count_tokens(table, None, Backend::Claude, Some(&md));
    assert!(
        count.count > 0,
        "empty-cell table should still produce tokens"
    );
}

#[test]
fn find_table_in_markdown() {
    let text = "Hello\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\nGoodbye\n";
    let md = MarkdownDecomposer;
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    let raw = count_tokens(text, None, Backend::Claude, None);
    assert!(
        aware.count >= raw.count,
        "table-aware ({}) should be >= raw ({})",
        aware.count,
        raw.count
    );
}

#[test]
fn claude_overcounts_vs_openai_with_tables() {
    let text = "# Report\n\n\
                | Metric | Value |\n\
                |--------|-------|\n\
                | CPU    | 85%   |\n\
                | Memory | 4 GB  |\n\n\
                Overall performance is satisfactory.";
    let md = MarkdownDecomposer;
    let claude = count_tokens(text, None, Backend::Claude, Some(&md));
    let openai = count_tokens(text, None, Backend::Openai, Some(&md));
    assert!(
        claude.count >= openai.count,
        "Claude ({}) should overcount vs OpenAI ({}) even with tables",
        claude.count,
        openai.count
    );
}

// ---------------------------------------------------------------------------
// Custom decomposer
// ---------------------------------------------------------------------------

/// A trivial custom decomposer that splits on newlines.
struct NewlineDecomposer;

impl Decomposer for NewlineDecomposer {
    fn count(&self, text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize {
        text.split('\n').map(raw_count).sum()
    }
}

#[test]
fn custom_decomposer_works() {
    let text = "line one\nline two\nline three";
    let d = NewlineDecomposer;
    let report = count_tokens(text, None, Backend::Claude, Some(&d));
    assert!(report.count > 0);
}
