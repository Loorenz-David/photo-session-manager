//! The OpenAI model adapter (R8·U1). Every provider-owned shape here comes from R1's captures
//! (`crates/research/fixtures/openai/`), never from memory or documentation prose.

pub mod adapter;
pub mod errors;
pub mod schema;
pub mod usage;

pub use adapter::OpenAiModel;
