use crate::{spool::Spool, sqlite::store::SqliteStore};
use axum::{Router, http::StatusCode, routing::any};
use psm_core::{Error, Result, ports::clock::SystemClock};
use std::{
    net::SocketAddr,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

pub struct NudgeServer {
    addr: SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl NudgeServer {
    pub fn start(spool: Arc<Spool>, store: Arc<SqliteStore>) -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel(1);
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
        let thread = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build();
            let Ok(runtime) = runtime else {
                let _ = tx.send(Err("runtime".to_string()));
                return;
            };
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await;
                let Ok(listener) = listener else {
                    let _ = tx.send(Err("bind".to_string()));
                    return;
                };
                let addr = listener.local_addr().unwrap();
                let _ = tx.send(Ok(addr));
                let app = Router::new().route(
                    "/nudge",
                    any(move || {
                        let spool = spool.clone();
                        let store = store.clone();
                        async move {
                            if let Err(e) = spool.drain(store.as_ref(), &SystemClock) {
                                tracing::error!(error=%e, "spool drain");
                            }
                            StatusCode::NO_CONTENT
                        }
                    }),
                );
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = stop_rx.await;
                    })
                    .await;
            });
        });
        let addr = rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|e| Error::Provider(e.to_string()))?
            .map_err(Error::Provider)?;
        Ok(Self {
            addr,
            stop: Some(stop_tx),
            thread: Some(thread),
        })
    }
    pub fn port(&self) -> u16 {
        self.addr.port()
    }
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for NudgeServer {
    fn drop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
