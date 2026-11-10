use crate::{
    Result,
    ports::{clock::Clock, editing::EditingProvider, store::Store},
    services::{
        arrival::ArrivalService,
        history::History,
        review::{ReviewResult, ReviewService},
        sessions::SessionsService,
    },
};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

pub struct PhotosService<'a> {
    pub arrival: &'a ArrivalService,
    pub editing: &'a dyn EditingProvider,
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub history: &'a History,
}
impl PhotosService<'_> {
    pub fn import(&self, paths: &[String]) -> Result<()> {
        let root = self.arrival.capture_root()?;
        let folder = match self.store.active_session_id()? {
            // A cleared item is read-only (10F Q3): an import goes to Unassigned instead.
            Some(id) if self.store.cleared_sessions()?.contains(&id) => "_Unassigned".into(),
            Some(id) => {
                let session = self.store.session(id)?;
                let item = self.store.item(
                    session
                        .session
                        .item_id
                        .ok_or(crate::Error::DocumentNotSession)?,
                )?;
                item.folder_key
            }
            None => "_Unassigned".into(),
        };
        let target = root.join(folder);
        fs::create_dir_all(&target)?;
        for path in paths {
            let source = Path::new(path);
            if !source.is_file() {
                return Err(crate::Error::InvalidValue(
                    "import path is not a file".into(),
                ));
            }
            let name = source
                .file_name()
                .ok_or_else(|| crate::Error::InvalidValue("import path has no name".into()))?;
            let temp = target.join(format!(
                ".{}.{}.tmp",
                name.to_string_lossy(),
                uuid::Uuid::new_v4()
            ));
            let result = (|| -> Result<PathBuf> {
                let mut input = File::open(source)?;
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temp)?;
                io::copy(&mut input, &mut output)?;
                output.sync_all()?;
                for n in 0.. {
                    let dest = if n == 0 {
                        target.join(name)
                    } else {
                        let base = Path::new(name)
                            .file_stem()
                            .unwrap_or(name)
                            .to_string_lossy();
                        let ext = Path::new(name)
                            .extension()
                            .map(|e| format!(".{}", e.to_string_lossy()))
                            .unwrap_or_default();
                        target.join(format!("{base}__dup{n}{ext}"))
                    };
                    match fs::hard_link(&temp, &dest) {
                        Ok(()) => return Ok(dest),
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                        Err(error) => return Err(error.into()),
                    }
                }
                unreachable!()
            })();
            let _ = fs::remove_file(&temp);
            self.arrival.import(&result?)?;
        }
        Ok(())
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
        ReviewService::new(self.store, self.clock, self.history).move_photos(&ids, target)
    }
}
