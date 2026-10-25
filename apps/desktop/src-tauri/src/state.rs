use psm_integrations::App;
use std::sync::Mutex;

pub struct AppState(pub Mutex<App>);
