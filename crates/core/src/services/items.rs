use crate::{
    Error, Result,
    domain::{
        item::{Item, NewItem},
        session::SessionDto,
    },
    mechanisms::{naming::folder_key, time::now_ms},
    ports::{
        clock::Clock,
        store::{Store, StoreTx},
    },
};

pub struct ItemsService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
}

impl<'a> ItemsService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self { store, clock }
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
        Ok(Some(format!("{prefix}-{next:04}")))
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
