use crate::{Result, domain::photo::Integrity, mechanisms::hash::sha256_file, ports::store::Store};
use std::path::Path;

pub struct IntegrityCheck<'a> {
    store: &'a dyn Store,
}

impl<'a> IntegrityCheck<'a> {
    pub fn new(store: &'a dyn Store) -> Self {
        Self { store }
    }
    pub fn verify(&self, photo_id: i64) -> Result<Integrity> {
        let photo = self.store.photo(photo_id)?;
        let integrity = match sha256_file(Path::new(&photo.raw_path)) {
            Ok(actual) if photo.raw_hash.as_deref() == Some(&actual) => Integrity::Ok,
            Ok(_) => Integrity::RawChanged,
            Err(crate::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                Integrity::RawMissing
            }
            Err(error) => return Err(error),
        };
        self.store.set_photo_integrity(photo_id, integrity)?;
        Ok(integrity)
    }
}
