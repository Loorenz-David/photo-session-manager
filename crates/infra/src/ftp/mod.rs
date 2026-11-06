pub mod names;
pub mod peers;
mod server;
mod session;
pub use server::{FtpReceiver, ReceiverHandle};

use ipnet::IpNet;
use psm_core::{
    Result,
    ports::{camera::CaptureSink, clock::Clock},
};
use std::{net::IpAddr, ops::RangeInclusive, path::PathBuf, sync::Arc};

pub const FTP_USERNAME: &str = "sessionmanager";
pub const DEFAULT_CONTROL_PORT: u16 = 2121;
pub const DEFAULT_PASSIVE: RangeInclusive<u16> = 50000..=50009;
pub use psm_core::ports::camera::CompletedNames;

#[derive(Clone)]
pub struct ReceiverConfig {
    pub bind: IpAddr,
    pub port: u16,
    pub passive: RangeInclusive<u16>,
    pub allowed_peers: Vec<IpNet>,
    pub username: String,
    pub password: String,
    pub capture_root: Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>,
    pub sink: Arc<dyn CaptureSink>,
    pub names: Arc<dyn CompletedNames>,
    pub clock: Arc<dyn Clock>,
}
impl ReceiverConfig {
    pub fn default_allowed_peers() -> Vec<IpNet> {
        [
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "169.254.0.0/16",
            "fc00::/7",
            "fe80::/10",
        ]
        .iter()
        .map(|n| n.parse().unwrap())
        .collect()
    }
}
pub(crate) fn ftp_error(e: impl std::fmt::Display) -> psm_core::Error {
    psm_core::Error::Provider(e.to_string())
}
pub(crate) type FtpResult<T> = Result<T>;
