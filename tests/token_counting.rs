#![allow(missing_docs)]

use ah_ah_ah::{count_tokens, Backend, Decomposer, MarkdownDecomposer};

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

// ---------------------------------------------------------------------------
// Fast-path heuristic: pipes in non-table content
// ---------------------------------------------------------------------------

#[test]
fn rust_match_arms_not_treated_as_table() {
    let code = r#"
match kind {
    Kind::Alpha => do_alpha(),
    Kind::Beta | Kind::Gamma => do_beta_gamma(),
    _ => Err(Error::Unknown),
}
"#;
    let md = MarkdownDecomposer;
    let raw = count_tokens(code, None, Backend::Claude, None);
    let aware = count_tokens(code, None, Backend::Claude, Some(&md));
    assert_eq!(
        aware.count, raw.count,
        "code with | match arms should use raw path (raw={}, aware={})",
        raw.count, aware.count
    );
}

#[test]
fn shell_pipe_not_treated_as_table() {
    let code = "cat file.txt | grep pattern | sort | uniq -c";
    let md = MarkdownDecomposer;
    let raw = count_tokens(code, None, Backend::Claude, None);
    let aware = count_tokens(code, None, Backend::Claude, Some(&md));
    assert_eq!(
        aware.count, raw.count,
        "shell pipes should use raw path (raw={}, aware={})",
        raw.count, aware.count
    );
}

#[test]
fn bitwise_or_not_treated_as_table() {
    let code = "let flags = FLAG_READ | FLAG_WRITE | FLAG_EXEC;";
    let md = MarkdownDecomposer;
    let raw = count_tokens(code, None, Backend::Claude, None);
    let aware = count_tokens(code, None, Backend::Claude, Some(&md));
    assert_eq!(
        aware.count, raw.count,
        "bitwise OR should use raw path (raw={}, aware={})",
        raw.count, aware.count
    );
}

#[test]
fn rust_closure_pipes_not_treated_as_table() {
    let code = r#"
items.iter()
    .filter(|item| item.is_valid())
    .map(|x| x.value)
    .collect::<Vec<_>>()
"#;
    let md = MarkdownDecomposer;
    let raw = count_tokens(code, None, Backend::Claude, None);
    let aware = count_tokens(code, None, Backend::Claude, Some(&md));
    assert_eq!(
        aware.count, raw.count,
        "closure pipes should use raw path (raw={}, aware={})",
        raw.count, aware.count
    );
}

// ---------------------------------------------------------------------------
// Tables without leading pipe (Finding 1 regression tests)
// ---------------------------------------------------------------------------

#[test]
fn table_without_leading_pipe_detected() {
    // pulldown-cmark accepts this as a valid pipe table.
    // The key assertion: decomposition IS applied (counts differ from raw),
    // meaning the heuristic correctly recognizes this as a table.
    let text = "Name | Age\n----|-----\nAlice | 30\nBob | 25\n";
    let md = MarkdownDecomposer;
    let raw = count_tokens(text, None, Backend::Claude, None);
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert_ne!(
        aware.count, raw.count,
        "table without leading pipe should trigger decomposition (raw={}, aware={})",
        raw.count, aware.count
    );
}

#[test]
fn table_without_outer_pipes_mixed_with_prose() {
    let text = "Some intro text.\n\nHeader A | Header B\n---------|--------\nfoo | bar\n\nSome trailing text.";
    let md = MarkdownDecomposer;
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert!(aware.count > 0, "should produce a positive count");
}

#[test]
fn separator_with_alignment_no_leading_pipe() {
    let text = "Left | Center | Right\n:-----|:------:|------:\na | b | c\n";
    let md = MarkdownDecomposer;
    let raw = count_tokens(text, None, Backend::Claude, None);
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert!(
        aware.count >= raw.count,
        "aligned table without leading pipe: aware ({}) should be >= raw ({})",
        aware.count,
        raw.count
    );
}

// ---------------------------------------------------------------------------
// Unicode and special characters
// ---------------------------------------------------------------------------

#[test]
fn unicode_emoji_tokens() {
    let text = "Hello 🌍🚀✨ world!";
    let claude = count_tokens(text, None, Backend::Claude, None);
    let openai = count_tokens(text, None, Backend::Openai, None);
    assert!(claude.count > 0);
    assert!(openai.count > 0);
}

#[test]
fn cjk_characters() {
    let text = "这是一个中文测试句子。日本語のテストです。한국어 테스트입니다.";
    let claude = count_tokens(text, None, Backend::Claude, None);
    let openai = count_tokens(text, None, Backend::Openai, None);
    assert!(claude.count > 0);
    assert!(openai.count > 0);
}

#[test]
fn mixed_scripts() {
    let text = "English 中文 العربية हिंदी 日本語";
    let claude = count_tokens(text, None, Backend::Claude, None);
    assert!(claude.count > 0);
}

#[test]
fn combining_characters() {
    // e + combining acute accent = é
    let text = "cafe\u{0301} nai\u{0308}ve";
    let claude = count_tokens(text, None, Backend::Claude, None);
    assert!(claude.count > 0);
}

#[test]
fn zero_width_characters() {
    let text = "a\u{200B}b\u{200C}c\u{200D}d\u{FEFF}e";
    let claude = count_tokens(text, None, Backend::Claude, None);
    assert!(claude.count > 0);
}

// ---------------------------------------------------------------------------
// Edge cases: whitespace and special inputs
// ---------------------------------------------------------------------------

#[test]
fn whitespace_only() {
    let claude = count_tokens("   ", None, Backend::Claude, None);
    let openai = count_tokens("   ", None, Backend::Openai, None);
    assert!(claude.count > 0);
    assert!(openai.count > 0);
}

#[test]
fn single_character() {
    let claude = count_tokens("x", None, Backend::Claude, None);
    assert!(claude.count > 0);
}

#[test]
fn newlines_only() {
    let claude = count_tokens("\n\n\n", None, Backend::Claude, None);
    assert!(claude.count > 0);
}

#[test]
fn tab_characters() {
    let claude = count_tokens("\t\t\t", None, Backend::Claude, None);
    assert!(claude.count > 0);
}

#[test]
fn very_long_word() {
    let word = "a".repeat(10_000);
    let claude = count_tokens(&word, None, Backend::Claude, None);
    let openai = count_tokens(&word, None, Backend::Openai, None);
    assert!(claude.count > 0);
    assert!(openai.count > 0);
}

#[test]
fn repeated_text_scales_linearly() {
    let unit = "The quick brown fox jumps over the lazy dog. ";
    let single = count_tokens(unit, None, Backend::Claude, None);
    let repeated = unit.repeat(10);
    let ten = count_tokens(&repeated, None, Backend::Claude, None);
    // Allow some slack for boundary effects but should be roughly 10x.
    assert!(
        ten.count >= single.count * 9,
        "10x text ({}) should be at least 9x single ({})",
        ten.count,
        single.count
    );
    assert!(
        ten.count <= single.count * 11,
        "10x text ({}) should be at most 11x single ({})",
        ten.count,
        single.count
    );
}

// ---------------------------------------------------------------------------
// Budget edge cases
// ---------------------------------------------------------------------------

#[test]
fn budget_exactly_at_count() {
    let text = "Hello";
    let count = count_tokens(text, None, Backend::Claude, None).count;
    let report = count_tokens(text, Some(count), Backend::Claude, None);
    assert!(
        !report.over_budget,
        "exactly at budget should not be over_budget"
    );
}

#[test]
fn budget_one_below_count() {
    let text = "Hello, world! This has several tokens.";
    let count = count_tokens(text, None, Backend::Claude, None).count;
    assert!(count > 1, "need a multi-token text for this test");
    let report = count_tokens(text, Some(count - 1), Backend::Claude, None);
    assert!(report.over_budget, "one below should be over_budget");
}

#[test]
fn no_budget_means_not_over_budget() {
    let report = count_tokens("anything", None, Backend::Claude, None);
    assert!(!report.over_budget);
    assert!(report.budget.is_none());
}

#[test]
fn budget_zero() {
    let report = count_tokens("x", Some(0), Backend::Claude, None);
    assert!(report.over_budget);
    assert_eq!(report.budget, Some(0));
}

// ---------------------------------------------------------------------------
// TokenReport serialization
// ---------------------------------------------------------------------------

#[test]
fn token_report_json_no_budget() {
    let report = count_tokens("Hi", None, Backend::Claude, None);
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("budget").is_none(), "budget should be skipped");
    assert!(json.get("count").is_some());
    assert!(json.get("over_budget").is_some());
    assert!(json.get("tokenizer").is_some());
}

#[test]
fn token_report_json_with_budget() {
    let report = count_tokens("Hi", Some(100), Backend::Claude, None);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["budget"], 100);
}

// ---------------------------------------------------------------------------
// Multi-table documents
// ---------------------------------------------------------------------------

#[test]
fn two_tables_in_one_document() {
    let text = "\
# Section 1

| A | B |
|---|---|
| 1 | 2 |

Some text between tables.

# Section 2

| X | Y | Z |
|---|---|---|
| a | b | c |
| d | e | f |
";
    let md = MarkdownDecomposer;
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    let raw = count_tokens(text, None, Backend::Claude, None);
    assert!(
        aware.count >= raw.count,
        "two-table aware ({}) should be >= raw ({})",
        aware.count,
        raw.count
    );
}

#[test]
fn table_with_alignment_colons() {
    let text = "\
| Left | Center | Right |
|:-----|:------:|------:|
| a    |   b    |     c |
";
    let md = MarkdownDecomposer;
    let aware = count_tokens(text, None, Backend::Claude, Some(&md));
    assert!(aware.count > 0, "aligned table should produce tokens");
}

// ---------------------------------------------------------------------------
// Decomposer with empty/trivial inputs
// ---------------------------------------------------------------------------

#[test]
fn decomposer_with_empty_text() {
    let md = MarkdownDecomposer;
    let report = count_tokens("", None, Backend::Claude, Some(&md));
    assert_eq!(report.count, 0);
}

#[test]
fn decomposer_with_whitespace_only() {
    let md = MarkdownDecomposer;
    let report = count_tokens("   \n\n  ", None, Backend::Claude, Some(&md));
    assert!(report.count > 0);
}

// ---------------------------------------------------------------------------
// Backend consistency: same text, same backend = same count
// ---------------------------------------------------------------------------

#[test]
fn deterministic_counting() {
    let text = "Token counting should be deterministic across calls.";
    let a = count_tokens(text, None, Backend::Claude, None);
    let b = count_tokens(text, None, Backend::Claude, None);
    assert_eq!(a.count, b.count, "same input should yield same count");

    let c = count_tokens(text, None, Backend::Openai, None);
    let d = count_tokens(text, None, Backend::Openai, None);
    assert_eq!(c.count, d.count, "same input should yield same count");
}

// ---------------------------------------------------------------------------
// OpenAI backend ignores decomposer (exact BPE guarantee)
// ---------------------------------------------------------------------------

#[test]
fn openai_ignores_decomposer() {
    let table = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let md = MarkdownDecomposer;
    let raw = count_tokens(table, None, Backend::Openai, None);
    let with_decomposer = count_tokens(table, None, Backend::Openai, Some(&md));
    assert_eq!(
        raw.count, with_decomposer.count,
        "OpenAI counts must be identical with or without decomposer (raw={}, decomposed={})",
        raw.count, with_decomposer.count
    );
}

#[test]
fn openai_exact_flag() {
    assert!(Backend::Openai.is_exact());
    assert!(!Backend::Claude.is_exact());
}

#[test]
fn openai_empty_text() {
    let report = count_tokens("", None, Backend::Openai, None);
    assert_eq!(report.count, 0);
}

#[test]
fn openai_single_char() {
    let report = count_tokens("a", None, Backend::Openai, None);
    assert_eq!(report.count, 1);
}

// ---------------------------------------------------------------------------
// API-validated fixture tests
//
// Expected counts come from the Anthropic count_tokens API (March 2026).
// See scripts/gen-token-fixtures.sh for how these were generated.
// ---------------------------------------------------------------------------

/// Assert that ah-ah-ah count matches the API count exactly.
fn assert_exact(text: &str, expected_api: usize) {
    let report = count_tokens(text, None, Backend::Claude, None);
    assert_eq!(
        report.count,
        expected_api,
        "exact mismatch: text={:?}, local={}, api={}",
        &text[..text.len().min(60)],
        report.count,
        expected_api
    );
}

/// Assert ah-ah-ah overcounts (local >= api). This is the safe direction
/// for budget enforcement. Panic if we undercount.
fn assert_overcount(text: &str, expected_api: usize) {
    let report = count_tokens(text, None, Backend::Claude, None);
    assert!(
        report.count >= expected_api,
        "UNDERCOUNT on Unicode/emoji: text={:?}, local={}, api={} (should overcount)",
        &text[..text.len().min(60)],
        report.count,
        expected_api
    );
}

/// Assert ah-ah-ah is within a tolerance band of the API count.
/// Small undercounts (-1 to -2) are acceptable on punctuation-heavy ASCII.
fn assert_near(text: &str, expected_api: usize, max_undercount: usize, max_overcount: usize) {
    let report = count_tokens(text, None, Backend::Claude, None);
    let lower = expected_api.saturating_sub(max_undercount);
    let upper = expected_api + max_overcount;
    assert!(
        report.count >= lower && report.count <= upper,
        "out of tolerance: text={:?}, local={}, api={}, allowed=[{}..{}]",
        &text[..text.len().min(60)],
        report.count,
        expected_api,
        lower,
        upper
    );
}

// --- Exact matches (delta = 0 against API) ---

#[test]
fn fixture_hello_world() {
    assert_exact("Hello, world!", 4);
}

#[test]
fn fixture_quick_brown_fox() {
    assert_exact("The quick brown fox jumps over the lazy dog.", 11);
}

#[test]
fn fixture_jwt_fragment() {
    assert_exact("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9", 31);
}

#[test]
fn fixture_rust_if_let() {
    // The fixture was generated with literal \n in bash (not real newlines).
    assert_exact(
        "    if let Some(x) = maybe_value {\\n        process(x);\\n    }",
        23,
    );
}

#[test]
fn fixture_sql_query() {
    assert_exact(
        "SELECT u.name, COUNT(*) FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.name;",
        30,
    );
}

#[test]
fn fixture_url() {
    assert_exact(
        "https://example.com/path/to/resource?query=value&other=123#fragment",
        21,
    );
}

#[test]
fn fixture_iso_timestamp() {
    assert_exact("2026-03-16T14:30:00.000Z", 15);
}

// --- Unicode/emoji: must overcount (safe direction for budgets) ---

#[test]
fn fixture_cjk_overcounts() {
    // API=7, local=13. CJK chars not in vocab → byte-level counting.
    assert_overcount("こんにちは世界", 7);
}

#[test]
fn fixture_emoji_overcounts() {
    // API=8, local=11. Emoji bytes not in vocab.
    assert_overcount("🌍🚀✨", 8);
}

// --- Near matches: small delta acceptable ---

#[test]
fn fixture_rust_fn_main() {
    // API=13, local=12 (delta -1). Merge-order divergence.
    assert_near(r#"fn main() { println!("Hello, world!"); }"#, 13, 2, 2);
}

#[test]
fn fixture_markdown_table() {
    // API=20, local=19 (delta -1).
    assert_near("| Name | Age |\\n|------|-----|\\n| Alice | 30 |", 20, 2, 2);
}

#[test]
fn fixture_price_string() {
    // API=21, local=19 (delta -2). Digits + $ signs.
    assert_near(
        "The price is $42.99 for 3 items (total: $128.97).",
        21,
        3,
        3,
    );
}

#[test]
fn fixture_http_request() {
    // API=22, local=21 (delta -1).
    assert_near("GET /api/v2/users?limit=100&offset=0 HTTP/1.1", 22, 2, 2);
}

#[test]
fn fixture_rust_error() {
    // API=19, local=18 (delta -1).
    assert_near(
        "error[E0308]: mismatched types expected `String`, found `&str`",
        19,
        2,
        2,
    );
}

#[test]
fn fixture_aho_corasick_sentence() {
    // API=24, local=23 (delta -1).
    assert_near(
        "The Aho-Corasick algorithm constructs a finite-state automaton from a set of patterns.",
        24,
        2,
        2,
    );
}

#[test]
fn fixture_json_object() {
    // API=28, local=26 (delta -2).
    assert_near(
        r#"{"name": "test", "values": [1, 2, 3], "nested": {"key": "value"}}"#,
        28,
        3,
        3,
    );
}

#[test]
fn fixture_latin_prose_overcounts() {
    // API=34, local=41 (delta +7). Latin words not fully in vocab.
    assert_overcount(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        34,
    );
}

#[test]
fn fixture_rust_fn_signature() {
    // API=31, local=33 (delta +2).
    assert_near(
        "    fn count(&self, text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize {",
        31,
        2,
        5,
    );
}
