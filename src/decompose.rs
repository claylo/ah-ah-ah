//! Text decomposition for boundary-aware token counting.
//!
//! Structured content (markdown tables, HTML, CSV) has boundaries that greedy
//! tokenizers shouldn't match across. A [`Decomposer`] defines how to identify
//! and split such regions so each piece is counted independently.

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Decomposes text for boundary-aware token counting.
///
/// When tokenizing structured content, a greedy tokenizer may match tokens
/// spanning structural boundaries (like `|` in markdown tables), leading to
/// undercounts. Implementors define how to find structured regions and count
/// them with boundary awareness.
///
/// # Example
///
/// ```
/// use ah_ah_ah::Decomposer;
///
/// struct PipeDecomposer;
///
/// impl Decomposer for PipeDecomposer {
///     fn count(&self, text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize {
///         // Split on pipes, count each segment independently.
///         let pipes = text.bytes().filter(|&b| b == b'|').count();
///         let segments: usize = text.split('|').map(|s| raw_count(s)).sum();
///         pipes + segments
///     }
/// }
/// ```
pub trait Decomposer: Send + Sync {
    /// Count tokens in text, respecting structural boundaries.
    ///
    /// `raw_count` is the underlying tokenizer's plain-text counter.
    /// Use it on individual segments after splitting at boundaries.
    fn count(&self, text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize;
}

/// Markdown-aware decomposer that respects table cell boundaries.
///
/// Uses `pulldown-cmark` to locate markdown tables, then splits each table
/// row on `|` so the tokenizer doesn't match tokens spanning cell boundaries.
/// Non-table text is passed through to the raw counter unchanged.
#[derive(Debug, Default, Clone)]
pub struct MarkdownDecomposer;

impl MarkdownDecomposer {
    /// Find byte ranges of markdown tables in the input.
    fn find_table_ranges(text: &str) -> Vec<Range<usize>> {
        let parser = Parser::new_ext(text, Options::ENABLE_TABLES).into_offset_iter();
        let mut ranges = Vec::new();
        let mut table_start: Option<usize> = None;

        for (event, range) in parser {
            match event {
                Event::Start(Tag::Table(_)) => {
                    table_start = Some(range.start);
                }
                Event::End(TagEnd::Table) => {
                    if let Some(start) = table_start.take() {
                        ranges.push(start..range.end);
                    }
                }
                _ => {}
            }
        }
        ranges
    }

    /// Cheap check for a markdown table separator row.
    ///
    /// Scans for lines matching the pattern `|` followed by `-` (with optional
    /// `:` for alignment), e.g. `|---|` or `| :--- |`. This avoids spinning up
    /// the pulldown-cmark parser on source code that merely contains pipes.
    fn has_table_separator(text: &str) -> bool {
        for line in text.split('\n') {
            let trimmed = line.trim();
            // A separator row must start with `|` and contain at least `|-`
            // or `|:-` (alignment colon before dashes).
            if trimmed.starts_with('|') && trimmed.len() >= 3 {
                // Check if the content after the first `|` is all separator
                // characters: `-`, `:`, `|`, space, tab.
                let rest = &trimmed[1..];
                let has_dash = rest.contains('-');
                let all_sep = rest
                    .bytes()
                    .all(|b| matches!(b, b'-' | b':' | b'|' | b' ' | b'\t'));
                if has_dash && all_sep {
                    return true;
                }
            }
        }
        false
    }

    /// Count tokens in a table fragment with cell-boundary awareness.
    ///
    /// Splits each line on `|`, counts each pipe as one token, and tokenizes
    /// cell contents independently.
    fn count_table(table_text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize {
        let mut count: usize = 0;
        for line in table_text.split('\n') {
            let pipes = line.bytes().filter(|&b| b == b'|').count();
            count += pipes;
            for segment in line.split('|') {
                count += raw_count(segment);
            }
        }
        count
    }
}

impl Decomposer for MarkdownDecomposer {
    fn count(&self, text: &str, raw_count: &dyn Fn(&str) -> usize) -> usize {
        // Fast path: no pipe character means no tables possible.
        if !text.contains('|') {
            return raw_count(text);
        }

        // Cheap heuristic: a real markdown table requires a separator row
        // containing `|` followed by dashes (optionally with colons for
        // alignment). Without one, pipes are just code — skip the parser.
        if !Self::has_table_separator(text) {
            return raw_count(text);
        }

        let table_ranges = Self::find_table_ranges(text);
        if table_ranges.is_empty() {
            return raw_count(text);
        }

        let mut count: usize = 0;
        let mut pos: usize = 0;

        for range in &table_ranges {
            // Non-table text before this table.
            if range.start > pos {
                count += raw_count(&text[pos..range.start]);
            }
            // Table region — cell-aware counting.
            count += Self::count_table(&text[range.start..range.end], raw_count);
            pos = range.end;
        }

        // Trailing non-table text.
        if pos < text.len() {
            count += raw_count(&text[pos..]);
        }

        count
    }
}
