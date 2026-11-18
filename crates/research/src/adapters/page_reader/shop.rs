//! The connected shop's own product pages, read through the commerce API instead of the
//! storefront (owner correction A9, 2026-11-18).
//!
//! The owner's shop sits behind Cloudflare, which answered 21 of 23 automated page reads with
//! "too many requests" (429, `Retry-After: 60`). A candidate on one of the shop's hosts whose
//! path names a product (`…/products/<handle>`, any locale or collection prefix) is read as that
//! product through [`CommerceAdapter::shop_product`]: title, type, vendor, the filled product
//! fields and the description become the page text, its JPEG images the page images. Every other
//! address goes to the wrapped reader. A handle the shop does not know is "could not be read",
//! with the reason; so is a failed API call (never an offline hold: the run goes on).

use std::sync::{Arc, Mutex};

use psm_core::ports::commerce::{CommerceAdapter, ShopProduct};
use serde_json::json;

use super::{MAX_TEXT_CHARS, http::MAX_IMAGES};
use crate::{
    ports::{CallFailure, Exchange, PageReader, PageReply},
    rules::sites::{host, overlaps, product_handle},
};

/// The longest edge asked of the shop's image service for a product image.
pub const SHOP_IMAGE_EDGE: u32 = 1600;

/// [`PageReader`] that reads the shop's own products through the API and every other page
/// through `inner`. It keeps `inner`'s id, so prices and call records are unchanged.
pub struct ShopPageReader {
    inner: Arc<dyn PageReader>,
    shop: Arc<dyn CommerceAdapter>,
    /// The shop's hosts (`H`), asked once; `None` until an answer arrived.
    hosts: Mutex<Option<Vec<String>>>,
}

impl ShopPageReader {
    pub fn new(inner: Arc<dyn PageReader>, shop: Arc<dyn CommerceAdapter>) -> Self {
        Self {
            inner,
            shop,
            hosts: Mutex::new(None),
        }
    }

    fn shop_hosts(&self) -> Vec<String> {
        let mut cached = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hosts) = cached.as_ref() {
            return hosts.clone();
        }
        let Ok(listing) = self.shop.public_listing("") else {
            return Vec::new();
        };
        let hosts: Vec<String> = listing
            .hosts
            .iter()
            .filter_map(|h| host(&format!("https://{}/", h.trim())))
            .collect();
        *cached = Some(hosts.clone());
        hosts
    }

    /// The product handle when `url` is a product page on one of the shop's hosts.
    fn own_product(&self, url: &str) -> Option<String> {
        let page_host = host(url)?;
        let handle = product_handle(url)?;
        let on_shop = self
            .shop_hosts()
            .iter()
            .any(|shop| overlaps(shop, &page_host));
        on_shop.then_some(handle)
    }

    fn unread(handle: &str, note: String, response: serde_json::Value) -> PageReply {
        PageReply {
            obtained: false,
            final_url: None,
            title: String::new(),
            text: String::new(),
            images: Vec::new(),
            usage: Some(Vec::new()),
            exchange: Exchange {
                request: json!({ "route": "shop_api", "handle": handle }),
                response,
            },
            blocked: false,
            note: Some(note),
        }
    }

    fn images(&self, product: &ShopProduct) -> Vec<Vec<u8>> {
        product
            .image_urls
            .iter()
            .take(MAX_IMAGES)
            .filter_map(|url| self.shop.fetch_image(url, SHOP_IMAGE_EDGE).ok())
            // The model adapter sends every image as `image/jpeg`, as the HTTP reader keeps them.
            .filter(|bytes| bytes.starts_with(&[0xFF, 0xD8, 0xFF]))
            .collect()
    }
}

/// The product as page text: what a storefront page says about it, field by field.
pub fn product_text(product: &ShopProduct) -> String {
    let mut lines = vec![product.title.clone()];
    if let Some(kind) = &product.item_type {
        lines.push(format!("Type: {kind}"));
    }
    if let Some(vendor) = &product.vendor {
        lines.push(format!("Vendor: {vendor}"));
    }
    for (label, value) in &product.fields {
        lines.push(format!("{label}: {value}"));
    }
    if let Some(description) = &product.description {
        lines.push(description.clone());
    }
    let text = lines.join("\n");
    text.chars().take(MAX_TEXT_CHARS).collect()
}

impl PageReader for ShopPageReader {
    fn id(&self) -> String {
        self.inner.id()
    }

    fn read(&self, url: &str) -> Result<PageReply, CallFailure> {
        let Some(handle) = self.own_product(url) else {
            return self.inner.read(url);
        };
        let product = match self.shop.shop_product(&handle) {
            Ok(Some(product)) => product,
            Ok(None) => {
                return Ok(Self::unread(
                    &handle,
                    "not found in your shop through Shopify (unpublished, renamed or deleted?)"
                        .into(),
                    json!({ "found": false }),
                ));
            }
            Err(error) => {
                return Ok(Self::unread(
                    &handle,
                    format!("Shopify did not answer: {error}"),
                    json!({ "error": error.to_string() }),
                ));
            }
        };
        let images = self.images(&product);
        Ok(PageReply {
            obtained: true,
            final_url: Some(product.url.clone().unwrap_or_else(|| url.to_owned())),
            title: product.title.clone(),
            text: product_text(&product),
            exchange: Exchange {
                request: json!({ "route": "shop_api", "handle": handle }),
                response: json!({
                    "found": true,
                    "url": product.url,
                    "title": product.title,
                    "fields": product.fields.len(),
                    "images": images.len(),
                }),
            },
            images,
            usage: Some(Vec::new()),
            blocked: false,
            note: None,
        })
    }
}
