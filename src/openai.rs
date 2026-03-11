//! OpenAI backend: exact o200k_base BPE encoding via bpe-openai.

/// Count tokens using the OpenAI o200k_base tokenizer (exact BPE).
pub fn count(text: &str) -> usize {
    let tokenizer = bpe_openai::o200k_base();
    tokenizer.count(text)
}
