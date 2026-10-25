use std::sync::Mutex;
use psm_integrations::App;

pub struct AppState(pub Mutex<App>);

