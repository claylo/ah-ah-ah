//! Tokenizer backend selection.

use serde::{Deserialize, Serialize};

/// Tokenizer backend for token counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    /// Claude 3+ (ctoc-verified vocab, greedy longest-match). Overcounts ~4%.
    #[default]
    Claude,
    /// OpenAI o200k_base (exact BPE encoding via bpe-openai).
    Openai,
}

impl Backend {
    /// Returns the backend name as a lowercase string slice.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Openai => "openai",
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
