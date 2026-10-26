//! The Shopify commerce adapter (ch09, 07A MC-10…MC-14, 09A MC-23/MC-24). Built here; wired into
//! the app from `settings.integration` by P13 task 0.

pub mod adapter;
pub mod auth;
pub mod errors;
pub mod graphql;
pub mod transport;
pub mod type_map;
pub mod types;

pub use adapter::{PollConfig, ShopifyAdapter, ShopifyConfig};
