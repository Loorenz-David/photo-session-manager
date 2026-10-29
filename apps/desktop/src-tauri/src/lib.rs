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
#[derive(serde::Serialize, Clone)]
struct PublishProgressPayload {
    session_id: i64,
    item_id: i64,
    step: psm_core::domain::session::PublishStep,
    uploaded: i64,
    total: i64,
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
            AppEvent::PublishProgress {
                session_id,
                item_id,
                step,
                uploaded,
                total,
            } => {
                let _ = self.0.emit(
                    "publish_progress",
                    PublishProgressPayload {
                        session_id,
                        item_id,
                        step,
                        uploaded,
                        total,
                    },
                );
            }
            AppEvent::HealthChanged => {
                let _ = self.0.emit("health_changed", ());
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
        commands::items::item_adopt,
        commands::items::item_update_identity,
        commands::items::items_search,
        commands::items::items_search_shop,
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
        commands::attributes::form_get,
        commands::publish::publish_preview,
        commands::publish::publish_confirm,
        commands::publish::publish_retry,
        commands::publish::notifications_list,
        commands::settings::settings_get,
        commands::settings::settings_set,
        commands::settings::crop_ratios_list
    ])
}

pub fn run() {
    let paths = AppPaths::from_env_or_default();
    let app = psm_integrations::App::open(paths).expect("open Session Manager data");
    let _logging = psm_infra::logging::init(&app.paths);
    let builder = specta_builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(app))
        .setup(|handle| {
            let state = handle.state::<AppState>();
            let previews = {
                let app = state.handles().expect("app state");
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
                        if let Ok(handles) = state.handles() {
                            handles.on_window_focus();
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

#[cfg(test)]
mod p17_tests {
    use super::*;
    use std::{
        fs,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        webview::InvokeRequest,
    };

    #[test]
    fn commands_are_async() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands");
        let mut command_count = 0;
        let mut async_count = 0;
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let code = fs::read_to_string(entry.path()).unwrap();
            command_count += code.matches("#[tauri::command]").count();
            async_count += code.matches("pub async fn ").count();
            assert!(
                !code.contains(".0.lock(")
                    && !code.contains(".0.read(")
                    && !code.contains(".0.write(")
            );
        }
        assert_eq!(command_count, async_count);
        assert!(command_count > 30);
    }

    fn request(cmd: &str, args: serde_json::Value) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    #[test]
    fn commands_do_not_wait_on_each_other() {
        let test = psm_testkit::TestApp::with_fake_document().unwrap();
        test.fake.state.lock().unwrap().delay_document_info_ms = 1_500;
        let psm_testkit::TestApp { dir: _dir, app, .. } = test;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::health::health_get,
                commands::sessions::sessions_list
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let webview = Arc::new(webview);
        let (tx, rx) = mpsc::channel();
        std::thread::scope(|scope| {
            let slow = webview.clone();
            let slow_tx = tx.clone();
            let slow_thread = scope.spawn(move || {
                let answer =
                    get_ipc_response(slow.as_ref(), request("health_get", serde_json::json!({})));
                slow_tx
                    .send(("health", answer.is_ok(), Instant::now()))
                    .unwrap();
            });
            std::thread::sleep(Duration::from_millis(100));
            let sent = Instant::now();
            let fast = get_ipc_response(
                webview.as_ref(),
                request(
                    "sessions_list",
                    serde_json::json!({"filter":"open","query":null}),
                ),
            );
            assert!(fast.is_ok());
            assert!(sent.elapsed() < Duration::from_millis(500));
            assert!(
                rx.try_recv().is_err(),
                "health_get should still be waiting for Capture One"
            );
            slow_thread.join().unwrap();
        });
        assert!(rx.recv().unwrap().1);
    }
}

#[cfg(test)]
mod p21_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn c2a_concurrent_settings_saves_preserve_every_change() {
        let test = psm_testkit::TestApp::new().unwrap();
        let path = test.app.paths.settings();
        let state = Arc::new(AppState::new(test.app));
        let patches = [
            json!({"preview_recipe":"R1"}),
            json!({"web_recipe":"W1"}),
            json!({"style_name":"S1"}),
            json!({"sku_prefix":"PX"}),
        ];
        let threads: Vec<_> = patches
            .into_iter()
            .map(|patch| {
                let state = state.clone();
                std::thread::spawn(move || state.settings_set(&patch))
            })
            .collect();
        for thread in threads {
            thread.join().unwrap().unwrap();
        }
        let shown = state.handles().unwrap().settings_get().unwrap();
        let stored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for (key, value) in [
            ("preview_recipe", "R1"),
            ("web_recipe", "W1"),
            ("style_name", "S1"),
            ("sku_prefix", "PX"),
        ] {
            assert_eq!(shown[key], value);
            assert_eq!(stored[key], value);
        }
    }
}
