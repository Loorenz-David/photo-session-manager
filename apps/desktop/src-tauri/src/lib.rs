mod commands;
mod state;

use tauri_specta::{collect_commands, Builder};
use state::AppState;
use psm_infra::paths::AppPaths;

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::health::health_get, commands::health::app_paths_get])
}

pub fn run() {
    let paths = AppPaths::from_env_or_default();
    let _logging = psm_infra::logging::init(&paths);
    let app = psm_integrations::App::open(paths).expect("open Session Manager data");
    let builder = specta_builder();
    tauri::Builder::default()
        .manage(AppState(std::sync::Mutex::new(app)))
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("run Session Manager");
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        super::specta_builder().export(specta_typescript::Typescript::default(), "../src/services/bindings.ts").unwrap();
    }
}

