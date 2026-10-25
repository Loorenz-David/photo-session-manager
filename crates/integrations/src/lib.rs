pub mod editing;

use psm_core::{Result, ports::clock::SystemClock};
use psm_infra::{
    nudge::NudgeServer, osascript::OsaRunner, paths::AppPaths, settings_file::Settings,
    spool::Spool, sqlite::store::SqliteStore, watcher::FsWatcher,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub struct App {
    pub paths: AppPaths,
    pub settings: Settings,
    pub store: Arc<SqliteStore>,
    pub spool: Arc<Spool>,
    pub nudge: NudgeServer,
    pub hooks_compiled: bool,
    stop: Arc<AtomicBool>,
    background: Option<thread::JoinHandle<()>>,
}

impl App {
    pub fn probe_capture_one(&self) -> Result<String> {
        let script = self.paths.hooks().join("probe_capture_one.js");
        std::fs::write(&script, "Application(\"Capture One\").name()")?;
        OsaRunner::new(&self.paths.root).run(&script, &[], Duration::from_secs(30))
    }

    pub fn open(paths: AppPaths) -> Result<Self> {
        paths.create()?;
        let settings = Settings::open(&paths)?;
        let store = Arc::new(
            SqliteStore::open(&paths).map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        let spool = Arc::new(Spool::new(paths.clone()));
        spool.drain(store.as_ref(), &SystemClock)?;
        let nudge = NudgeServer::start(spool.clone(), store.clone())?;
        let hooks_compiled = editing::capture_one::hooks::compile_all(&paths, nudge.port()).is_ok();
        let stop = Arc::new(AtomicBool::new(false));
        let bg_stop = stop.clone();
        let bg_spool = spool.clone();
        let bg_store = store.clone();
        let spool_dir = paths.spool();
        let background = thread::spawn(move || {
            let Ok((_watcher, mut rx)) = FsWatcher::watch(&spool_dir) else {
                return;
            };
            let mut last_drain = std::time::Instant::now();
            while !bg_stop.load(Ordering::Relaxed) {
                if rx.try_recv().is_ok() || last_drain.elapsed() >= Duration::from_secs(5) {
                    let _ = bg_spool.drain(bg_store.as_ref(), &SystemClock);
                    last_drain = std::time::Instant::now();
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
        Ok(Self {
            paths,
            settings,
            store,
            spool,
            nudge,
            hooks_compiled,
            stop,
            background: Some(background),
        })
    }
    pub fn nudge_port(&self) -> u16 {
        self.nudge.port()
    }
    pub fn nudge_addr(&self) -> std::net::SocketAddr {
        self.nudge.addr()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(bg) = self.background.take() {
            let _ = bg.join();
        }
    }
}
