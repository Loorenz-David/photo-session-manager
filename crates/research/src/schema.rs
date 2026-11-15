//! `psm_research::schema::ensure`: the engine's five tables in the app database file, created by
//! an idempotent step outside the numbered migration list (`master_plan_research.md` §6.4). Run
//! by whoever opens the store for research.

use psm_core::Result;
use psm_infra::sqlite::store::SqliteStore;

pub fn ensure(store: &SqliteStore) -> Result<()> {
    store.research_transaction(|tx| crate::store::create_tables(tx.conn()))
}
