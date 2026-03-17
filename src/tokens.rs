//! Public token counting API.

use serde::{Deserialize, Serialize};

use crate::backend::Backend;
use crate::decompose::Decomposer;
use crate::{claude, openai};

/// Result of counting tokens in a text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenReport {
    /// Number of tokens in the text.
    pub count: usize,
    /// Token budget (if provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<usize>,
    /// Whether the count exceeds the budget.
    pub over_budget: bool,
    /// Which tokenizer backend produced this count.
    pub tokenizer: String,
}

/// Count tokens in text using the specified backend.
///
/// # Arguments
///
/// * `text` — The text to tokenize.
/// * `budget` — Optional maximum token count. If provided, `over_budget`
///   in the report indicates whether the text exceeds it.
/// * `backend` — Which tokenizer to use.
/// * `decomposer` — Optional decomposer for structured content. When provided,
///   the decomposer handles boundary-aware counting (e.g., markdown tables).
///   Pass `None` for raw counting.
#[tracing::instrument(skip(text, decomposer), fields(text_len = text.len(), backend = %backend))]
pub fn count_tokens(
    text: &str,
    budget: Option<usize>,
    backend: Backend,
    decomposer: Option<&dyn Decomposer>,
) -> TokenReport {
    let raw_count: fn(&str) -> usize = match backend {
        Backend::Claude => claude::count_raw,
        Backend::Openai => openai::count,
    };

    // Exact backends (BPE) don't benefit from decomposition — applying one
    // would break cell contents into segments that tokenize differently,
    // violating the exactness guarantee.
    let effective_decomposer = if backend.is_exact() {
        if decomposer.is_some() {
            tracing::debug!("skipping decomposer for exact backend {backend}");
        }
        None
    } else {
        decomposer
    };

    let count = effective_decomposer.map_or_else(|| raw_count(text), |d| d.count(text, &raw_count));

    let over_budget = budget.is_some_and(|max| count > max);

    TokenReport {
        count,
        budget,
        over_budget,
        tokenizer: backend.as_str().to_string(),
    }
}
