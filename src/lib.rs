//! ah-ah-ah
//!
//! Offline token counting with pluggable backends and boundary-aware
//! decomposition.
//!
//! Two backends are available:
//!
//! - **Claude** (default): Uses ctoc's 38,360 API-verified Claude 3+ tokens
//!   with greedy longest-match via Aho-Corasick. Overcounts by ~4% compared
//!   to the real Claude tokenizer — safe for budget enforcement.
//! - **OpenAI**: Uses `bpe-openai` for exact o200k_base BPE encoding.
//!
//! Structured content (markdown tables, etc.) can cause greedy tokenizers to
//! match tokens spanning structural boundaries. The [`Decomposer`] trait lets
//! you plug in boundary-aware counting. A [`MarkdownDecomposer`] is included.
//!
//! # Quick start
//!
//! ```
//! use ah_ah_ah::{count_tokens, Backend, MarkdownDecomposer};
//!
//! // Raw counting (no decomposition).
//! let report = count_tokens("Hello, world!", None, Backend::Claude, None);
//! assert!(report.count > 0);
//!
//! // With markdown-aware decomposition.
//! let md = MarkdownDecomposer;
//! let report = count_tokens("| A | B |\n|---|---|\n| 1 | 2 |", None, Backend::Claude, Some(&md));
//! assert!(report.count > 0);
//! ```
#![deny(unsafe_code)]

pub mod backend;
mod claude;
pub mod decompose;
pub mod error;
mod openai;
pub mod tokens;

pub use backend::Backend;
pub use decompose::{Decomposer, MarkdownDecomposer};
pub use error::{Error, Result};
pub use tokens::{TokenReport, count_tokens};
