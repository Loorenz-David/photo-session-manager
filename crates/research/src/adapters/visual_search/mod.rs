//! Visual-search adapters (R8·U2). Google Lens through SerpApi is the owner's pick (evidence doc
//! "Owner picks" 1); the port stays as it is, so Cloud Vision can be added beside it later.

pub mod serpapi_lens;

pub use serpapi_lens::SerpApiLens;
