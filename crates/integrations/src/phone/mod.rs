//! The phone remote (book 11 §Phone remote, 10J; `master_plan_live.md` §5 PH): pairing, the Mac's
//! name and addresses for the QR code, and (LV2·U4–U5) the embedded HTTP + WebSocket server over
//! the same core services as the Mac's commands.
pub mod auth;
pub mod host;
pub mod pairing;
pub(crate) mod routes;
pub mod server;
pub mod web_root;
pub mod wire;
pub mod ws;
