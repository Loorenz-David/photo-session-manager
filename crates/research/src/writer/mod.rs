//! The publication writer (book 08C; `master_plan_writer.md`). W1 builds the product note: which
//! note counts, the owner's decisions over it, its staleness and **Look again** (`note`), and the
//! app-facing surface (`api`). W2 builds the writer run behind the crate's fakes: its settings
//! (`config`) and shipped guide (`defaults`), what it receives (`input`), the post-check
//! (`postcheck`), the run's life and call (`run`, `schema`), its writes (`writes`) and the
//! safeguard (`safeguard`).

pub mod api;
pub mod config;
pub mod defaults;
pub mod input;
pub mod note;
