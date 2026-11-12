//! Shop facts for a run (MC-A3): the connected shop's public hosts and the item's own storefront
//! listing, read through core's `CommerceAdapter::public_listing` and normalised by `H`.

use std::sync::Arc;

use psm_core::{Result, ports::commerce::CommerceAdapter};

use crate::rules::sites::{OwnListing, host};

/// What a run snapshots about the shop.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShopFacts {
    pub hosts: Vec<String>,
    pub own_listing: OwnListing,
}

pub struct CommerceShopFacts {
    adapter: Arc<dyn CommerceAdapter>,
}

/// `H` of a bare host or a URL; `None` when it has no valid host.
fn normalised_host(text: &str) -> Option<String> {
    let text = text.trim();
    if text.contains("://") {
        host(text)
    } else {
        host(&format!("https://{text}/"))
    }
}

impl CommerceShopFacts {
    pub fn new(adapter: Arc<dyn CommerceAdapter>) -> Self {
        Self { adapter }
    }

    /// The shop's hosts, and the item's own listing when it is linked (`external_id`).
    pub fn facts(&self, external_id: Option<&str>) -> Result<ShopFacts> {
        let listing = self.adapter.public_listing(external_id.unwrap_or(""))?;
        let mut hosts: Vec<String> = Vec::new();
        for h in listing.hosts.iter().filter_map(|h| normalised_host(h)) {
            if !hosts.contains(&h) {
                hosts.push(h);
            }
        }
        let own_listing = match external_id {
            Some(_) => OwnListing {
                url: listing.item_url.clone(),
                path_tail: listing
                    .item_path_tail
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
            },
            None => OwnListing::default(),
        };
        Ok(ShopFacts { hosts, own_listing })
    }
}
