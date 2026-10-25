mod commands;
mod state;

use psm_infra::paths::AppPaths;
use state::AppState;
use tauri_specta::{Builder, collect_commands};

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_get,
        commands::health::app_paths_get,
        commands::items::item_create,
        commands::items::item_update_identity,
        commands::sessions::session_create,
        commands::sessions::session_get,
        commands::sessions::sessions_list,
        commands::photos::photos_classify,
        commands::photos::photos_reorder,
        commands::photos::photos_delete,
        commands::photos::photos_undo,
        commands::photos::photos_move,
        commands::photos::photos_copy
    ])
}

pub fn run() {
    let paths = AppPaths::from_env_or_default();
    let app = psm_integrations::App::open(paths).expect("open Session Manager data");
    let _logging = psm_infra::logging::init(&app.paths);
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
        super::specta_builder()
            .export(
                specta_typescript::Typescript::default(),
                "../src/services/bindings.ts",
            )
            .unwrap();
    }
}
