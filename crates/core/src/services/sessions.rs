use crate::{
    Result,
    domain::session::SessionDto,
    mechanisms::time::now_ms,
    ports::{clock::Clock, store::Store},
};

pub struct SessionsService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
}

impl<'a> SessionsService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self { store, clock }
    }

    pub fn create(&self, item_id: i64) -> Result<SessionDto> {
        let mut id = 0;
        let at = now_ms(self.clock);
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            id = tx.insert_session(item_id, at)?;
            Ok(())
        })?;
        self.store.session(id)
    }

    pub fn get(&self, id: i64) -> Result<SessionDto> {
        self.store.session(id)
    }
    pub fn unassigned(&self) -> Result<SessionDto> {
        self.get(1)
    }
}
