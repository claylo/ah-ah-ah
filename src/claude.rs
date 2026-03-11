//! Claude backend: greedy longest-match on ctoc's verified vocabulary.
//!
//! Uses 38,360 API-verified Claude 3+ token strings with an Aho-Corasick
//! automaton for greedy leftmost-longest matching. Overcounts by ~4% compared
//! to the real Claude tokenizer — safe for budget enforcement.
//!
//! For exact Claude counts, use the Anthropic `count_tokens` API.

use std::sync::LazyLock;

use aho_corasick::AhoCorasick;

/// The 38,360 API-verified Claude 3+ token strings from ctoc.
static CLAUDE_VOCAB_JSON: &str = include_str!("claude_vocab.json");

/// Pre-built Aho-Corasick automaton for greedy longest-match tokenization.
static CLAUDE_AUTOMATON: LazyLock<AhoCorasick> = LazyLock::new(|| {
    let vocab: Vec<String> =
        serde_json::from_str(CLAUDE_VOCAB_JSON).expect("embedded claude_vocab.json is valid");
    AhoCorasick::builder()
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(&vocab)
        .expect("aho-corasick build should succeed for verified vocab")
});

/// Raw greedy longest-match token count (no structural awareness).
///
/// Walks the input left-to-right, greedily matching the longest known token
/// at each position. Unmatched bytes are counted as one token each
/// (conservative — these are characters not in the known vocab).
pub fn count_raw(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut count: usize = 0;
    let mut pos: usize = 0;
    let bytes = text.as_bytes();

    for mat in CLAUDE_AUTOMATON.find_iter(text) {
        // Count any unmatched bytes before this match as individual tokens.
        count += mat.start() - pos;
        // Count the matched token.
        count += 1;
        pos = mat.end();
    }

    // Count any trailing unmatched bytes.
    count += bytes.len() - pos;
    count
}
