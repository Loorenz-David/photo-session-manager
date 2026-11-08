use super::{FtpResult, ReceiverConfig, ftp_error, session};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::watch};

pub(crate) struct Shared {
    pub config: ReceiverConfig,
    pub active: Mutex<HashSet<String>>,
    pub uploads: AtomicUsize,
}

pub struct FtpReceiver;
pub struct ReceiverHandle {
    port: u16,
    shared: Arc<Shared>,
    stop: Option<watch::Sender<bool>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl FtpReceiver {
    pub fn start(config: ReceiverConfig) -> FtpResult<ReceiverHandle> {
        let shared = Arc::new(Shared {
            config,
            active: Mutex::new(HashSet::new()),
            uploads: AtomicUsize::new(0),
        });
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let (stop_tx, mut stop_rx) = watch::channel(false);
        let worker = shared.clone();
        let thread = thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };
            runtime.block_on(async move {
                let addr = SocketAddr::new(worker.config.bind, worker.config.port);
                let listener = match TcpListener::bind(addr).await {
                    Ok(l) => l, Err(e) => { let _ = ready_tx.send(Err(e.to_string())); return; }
                };
                let _ = ready_tx.send(listener.local_addr().map(|a| a.port()).map_err(|e| e.to_string()));
                // Completed markers survive a stopped receiver. The ledger's first-copy rule
                // makes a retry safe if a prior delivery succeeded before the stop.
                if let Some(root) = (worker.config.capture_root)() {
                    let incoming = root.join(".incoming");
                    if let Ok(mut entries) = tokio::fs::read_dir(incoming).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let path = entry.path();
                            let Some(name) = path.file_name().and_then(|n| n.to_str())
                                .and_then(|n| n.strip_prefix('.'))
                                .and_then(|n| n.strip_suffix(".done")) else { continue };
                            if let Ok(meta) = entry.metadata().await {
                                session::file_completed(path.clone(), name.to_owned(), meta.len(), worker.config.clock.now_ms(), worker.clone()).await;
                            }
                        }
                    }
                }
                loop {
                    tokio::select! {
                        _ = stop_rx.changed() => break,
                        accept = listener.accept() => {
                            let Ok((mut stream, peer)) = accept else { break };
                            if !super::peers::is_allowed(peer.ip(), &worker.config.allowed_peers) {
                                let _ = stream.write_all(b"421 Connections from this network are not allowed.\r\n").await;
                                continue;
                            }
                            let state = worker.clone();
                            let stop = stop_rx.clone();
                            tokio::spawn(async move { let _ = session::serve(stream, peer, state, stop).await; });
                        }
                    }
                }
            });
        });
        let port = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(ftp_error)?
            .map_err(ftp_error)?;
        Ok(ReceiverHandle {
            port,
            shared,
            stop: Some(stop_tx),
            thread: Some(thread),
        })
    }
}
impl ReceiverHandle {
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn uploads_in_progress(&self) -> usize {
        self.shared.uploads.load(Ordering::SeqCst)
    }
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(true);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for ReceiverHandle {
    fn drop(&mut self) {
        self.stop();
    }
}
