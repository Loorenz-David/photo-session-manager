pub mod client;
pub mod protocol;
pub mod supervisor;
pub const RESTART_BACKOFF_S: [u64; 5] = [1, 2, 4, 8, 16];
pub const RESTART_STEADY_S: u64 = 30;
pub const QUIT_GRACE_MS: u64 = 3_000;
