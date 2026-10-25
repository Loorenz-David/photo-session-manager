mod commands;
mod state;

use psm_core::{events::AppEvent, ports::events::EventSink};
use psm_infra::paths::AppPaths;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

struct TauriEvents(tauri::AppHandle);
#[derive(serde::Serialize, Clone)]
struct PhotoAddedPayload {
    photo_id: i64,
    session_id: i64,
}
#[derive(serde::Serialize, Clone)]
struct SessionChangedPayload {
    session_id: i64,
}
#[derive(serde::Serialize, Clone)]
struct ExportProgressPayload {
    session_id: i64,
    done: usize,
    total: usize,
    state: psm_core::events::ExportState,
}
impl EventSink for TauriEvents {
    fn emit(&self, event: AppEvent) {
        match event {
            AppEvent::PhotoAdded {
                photo_id,
                session_id,
            } => {
                let _ = self.0.emit(
                    "photo_added",
                    PhotoAddedPayload {
                        photo_id,
                        session_id,
                    },
                );
            }
            AppEvent::PhotoChanged {
                photo_id,
                session_id,
            } => {
                let _ = self.0.emit(
                    "photo_changed",
                    PhotoAddedPayload {
                        photo_id,
                        session_id,
                    },
                );
            }
            AppEvent::SessionChanged { session_id } => {
                let _ = self
                    .0
                    .emit("session_changed", SessionChangedPayload { session_id });
            }
            AppEvent::ExportProgress {
                session_id,
                done,
                total,
                state,
            } => {
                let _ = self.0.emit(
                    "export_progress",
                    ExportProgressPayload {
                        session_id,
                        done,
                        total,
                        state,
                    },
                );
            }
            _ => {}
        }
    }
}
use tauri_specta::{Builder, collect_commands};

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_get,
        commands::health::app_paths_get,
        commands::items::item_create,
        commands::items::item_update_identity,
        commands::items::items_search,
        commands::items::items_next_sku,
        commands::sessions::session_create,
        commands::sessions::session_get,
        commands::sessions::sessions_list,
        commands::sessions::session_activate,
        commands::sessions::session_deactivate,
        commands::sessions::session_active,
        commands::camera::camera_status,
        commands::camera::camera_retry,
        commands::camera::camera_capture,
        commands::photos::photos_classify,
        commands::photos::photos_reorder,
        commands::photos::photos_delete,
        commands::photos::photos_undo,
        commands::photos::photos_move,
        commands::photos::photos_copy,
        commands::photos::photos_import,
        commands::photos::photos_assign_unassigned,
        commands::export::export_run,
        commands::export::export_open_folder,
        commands::export::integration_get,
        commands::attributes::attributes_definitions_list,
        commands::attributes::attribute_values_get,
        commands::attributes::attribute_values_set,
        commands::attributes::attribute_ai_accept_all,
        commands::attributes::attribute_ai_clear,
        commands::attributes::presets_get,
        commands::attributes::presets_set,
        commands::attributes::form_get
    ])
}

pub fn run() {
    let paths = AppPaths::from_env_or_default();
    let app = psm_integrations::App::open(paths).expect("open Session Manager data");
    let _logging = psm_infra::logging::init(&app.paths);
    let builder = specta_builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(std::sync::Mutex::new(app)))
        .setup(|handle| {
            let state = handle.state::<AppState>();
            let previews = {
                let app = state.0.lock().expect("app state");
                app.events
                    .set_target(Arc::new(TauriEvents(handle.handle().clone())));
                app.paths.root.join("previews")
            };
            // The workspace loads preview files through the asset protocol; only the previews folder is readable.
            handle
                .asset_protocol_scope()
                .allow_directory(previews, true)?;
            if let Some(window) = handle.get_webview_window("main") {
                let app_handle = handle.handle().clone();
                window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Focused(true)) {
                        let state = app_handle.state::<AppState>();
                        if let Ok(app) = state.0.lock() {
                            let _ = app.previews.on_focus();
                        }
                    }
                });
            }
            Ok(())
        })
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
