use crate::{
    Error, Result,
    domain::{
        item::{Item, ItemOrigin, NewItem},
        session::SessionDto,
    },
    mechanisms::{naming::folder_key, time::now_ms},
    ports::{
        clock::Clock,
        commerce::CommerceAdapter,
        store::{Store, StoreTx},
    },
};

pub struct ItemsService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    commerce: Option<&'a dyn CommerceAdapter>,
}

impl<'a> ItemsService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self {
            store,
            clock,
            commerce: None,
        }
    }

    pub fn with_commerce(mut self, commerce: Option<&'a dyn CommerceAdapter>) -> Self {
        self.commerce = commerce;
        self
    }

    /// Local items whose SKU or title contains `query` (case-insensitive). The commerce adapter's
    /// matches join this list once an integration exists (P12); until then local items only.
    pub fn search(&self, query: &str) -> Result<Vec<crate::domain::item::Item>> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        self.store.transaction(&mut |tx| {
            items = tx
                .items()?
                .into_iter()
                .filter(|item| {
                    item.sku.to_lowercase().contains(&needle)
                        || item.title.to_lowercase().contains(&needle)
                })
                .collect();
            Ok(())
        })?;
        if let Some(adapter) = self.commerce
            && let Ok(hits) = adapter.item_find(query)
        {
            let links = self.store.all_item_links("shopify")?;
            for hit in hits {
                if links.iter().any(|link| link.external_id == hit.external_id) {
                    continue;
                }
                let Some(sku) = hit.sku else {
                    continue;
                };
                if items.iter().any(|item| item.sku == sku) {
                    continue;
                }
                items.push(Item {
                    id: 0,
                    folder_key: folder_key(&sku)?,
                    sku,
                    barcode: hit.barcode,
                    item_type: String::new(),
                    title: hit.title,
                    origin: ItemOrigin::Linked,
                    external_id: Some(hit.external_id),
                    created_at: 0,
                    updated_at: 0,
                });
            }
        }
        Ok(items)
    }

    /// MC-22 proposal: `<prefix>-<NNNN>` with `NNNN` = 1 + the largest `n` among local SKUs matching
    /// `^<prefix>-(\d+)$` exactly (case-sensitive), or 1; zero-padded to at least 4 digits.
    /// `prefix` is the `sku_prefix` setting; absent or empty → no proposal.
    pub fn next_sku(&self, prefix: Option<&str>) -> Result<Option<String>> {
        let Some(prefix) = prefix.filter(|p| !p.is_empty()) else {
            return Ok(None);
        };
        let mut largest = 0_u64;
        self.store.transaction(&mut |tx| {
            for item in tx.items()? {
                let number = item
                    .sku
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.strip_prefix('-'))
                    .filter(|digits| {
                        !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
                    })
                    .and_then(|digits| digits.parse::<u64>().ok());
                if let Some(number) = number {
                    largest = largest.max(number);
                }
            }
            Ok(())
        })?;
        let next = largest
            .checked_add(1)
            .ok_or_else(|| Error::InvalidValue("SKU sequence exhausted".into()))?;
        let local = format!("{prefix}-{next:04}");
        if let Some(adapter) = self.commerce {
            for offset in 0..50_u64 {
                let candidate = format!("{prefix}-{:04}", next + offset);
                match adapter.item_search(&candidate) {
                    Ok(found) if found.is_empty() => return Ok(Some(candidate)),
                    Err(_) => return Ok(Some(local)),
                    _ => {}
                }
            }
        }
        Ok(Some(local))
    }

    fn check_shop_sku(&self, sku: &str, item_id: Option<i64>) -> Result<()> {
        let Some(adapter) = self.commerce else {
            return Ok(());
        };
        let Ok(found) = adapter.item_search(sku) else {
            return Ok(());
        };
        let own = item_id
            .and_then(|id| self.store.item_link(id, "shopify").ok().flatten())
            .map(|link| link.external_id);
        if found
            .iter()
            .any(|hit| Some(&hit.external_id) != own.as_ref())
        {
            return Err(Error::SkuInShop { sku: sku.into() });
        }
        Ok(())
    }

    pub fn adopt(&self, external_id: &str) -> Result<Item> {
        let adapter = self.commerce.ok_or(Error::NoIntegration)?;
        let hit = adapter.item_get(external_id)?;
        let sku = hit
            .sku
            .ok_or_else(|| Error::InvalidValue("shop item has no SKU".into()))?;
        let key = folder_key(&sku)?;
        let mut id = 0;
        let at = now_ms(self.clock);
        self.store.transaction(&mut |tx| {
            Self::collision(tx, None, &sku, &key)?;
            id = tx.insert_item(NewItem {
                sku: &sku,
                folder_key: &key,
                barcode: hit.barcode.as_deref(),
                item_type: "",
                title: &hit.title,
                origin: "linked",
                at,
            })?;
            tx.insert_item_link(id, "shopify", external_id, None)
        })?;
        self.store.item(id)
    }

    fn collision(tx: &dyn StoreTx, item_id: Option<i64>, sku: &str, key: &str) -> Result<()> {
        for other in tx.items()? {
            if Some(other.id) == item_id {
                continue;
            }
            if other.sku == sku
                || other.folder_key.eq_ignore_ascii_case(key)
                || folder_key(&other.sku)?.eq_ignore_ascii_case(key)
            {
                return Err(Error::SkuCollision {
                    existing_sku: other.sku,
                });
            }
        }
        Ok(())
    }

    pub fn create_local(
        &self,
        sku: &str,
        title: &str,
        item_type: &str,
        barcode: Option<&str>,
    ) -> Result<(Item, SessionDto)> {
        let key = folder_key(sku)?;
        self.check_shop_sku(sku, None)?;
        let at = now_ms(self.clock);
        let mut item_id = 0;
        let mut session_id = 0;
        self.store.transaction(&mut |tx| {
            Self::collision(tx, None, sku, &key)?;
            item_id = tx.insert_item(NewItem {
                sku,
                folder_key: &key,
                barcode,
                item_type,
                title,
                origin: "local",
                at,
            })?;
            session_id = tx.insert_session(item_id, at)?;
            crate::services::attributes::prefill_defaults(tx, item_id, item_type, at)?;
            Ok(())
        })?;
        Ok((self.store.item(item_id)?, self.store.session(session_id)?))
    }

    pub fn link(
        &self,
        item_id: i64,
        integration: &str,
        external_id: &str,
        external_url: Option<&str>,
    ) -> Result<()> {
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            tx.insert_item_link(item_id, integration, external_id, external_url)
        })
    }

    pub fn update_identity(
        &self,
        item_id: i64,
        sku: Option<&str>,
        barcode: Option<&str>,
    ) -> Result<Item> {
        let at = now_ms(self.clock);
        if let Some(next) = sku {
            self.check_shop_sku(next, Some(item_id))?;
        }
        self.store.transaction(&mut |tx| {
            let old = tx.item(item_id)?;
            let next_sku = sku.unwrap_or(&old.sku);
            let proposed_key = folder_key(next_sku)?;
            Self::collision(tx, Some(item_id), next_sku, &proposed_key)?;
            let stored_key = if tx.item_has_photos(item_id)? {
                old.folder_key
            } else {
                proposed_key
            };
            tx.update_item_identity(
                item_id,
                next_sku,
                &stored_key,
                barcode.or(old.barcode.as_deref()),
                at,
            )
        })?;
        self.store.item(item_id)
    }
}
