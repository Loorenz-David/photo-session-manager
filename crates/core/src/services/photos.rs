use crate::{
    Result,
    ports::{clock::Clock, editing::EditingProvider, store::Store},
    services::{
        arrival::ArrivalService,
        review::{ReviewResult, ReviewService},
        sessions::SessionsService,
    },
};
use std::path::Path;

pub struct PhotosService<'a> {
    pub arrival: &'a ArrivalService,
    pub editing: &'a dyn EditingProvider,
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
}
impl PhotosService<'_> {
    pub fn import(&self, paths: &[String]) -> Result<()> {
        for path in paths {
            self.arrival.import(Path::new(path))?;
        }
        self.editing.import(paths)
    }
    pub fn assign_unassigned(&self, item_id: i64) -> Result<ReviewResult> {
        self.store.item(item_id)?;
        let target = match self.store.session_for_item(item_id)? {
            Some(id) => id,
            None => {
                SessionsService::new(self.store, self.clock)
                    .create(item_id)?
                    .session
                    .id
            }
        };
        let ids: Vec<_> = self
            .store
            .photos_in_session(1)?
            .iter()
            .map(|p| p.id)
            .collect();
        ReviewService::new(self.store, self.clock).move_photos(&ids, target)
    }
}
