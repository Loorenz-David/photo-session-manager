//! PH3–PH4: the phone's HTTP routes. Handlers parse, check the token and the session guard, call
//! the core services on a blocking thread and serialize (H8: no rule lives here). Every API
//! request logs `phone.request route= status= pairing=` (never the token).
use std::{
    net::{IpAddr, SocketAddr},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, Path as UrlPath, RawQuery, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use psm_core::{
    Error, Result,
    domain::{
        composition::Shape,
        live::{FocusPoint, PhonePairing},
    },
    ports::store::Store,
    services::{
        camera::CameraService,
        current_shot::Current,
        live_view,
        phone_view::{self, phone_view},
        review::ReviewService,
        template_placement::{self, HandAction, MoveTarget},
    },
};

use crate::{
    AppHandles,
    phone::{
        auth::token_of,
        host::mac_name,
        pairing::PhonePairings,
        server::Shared,
        wire::{
            self, CurrentBody, FEATURES, FocusBody, Hello, MoveBody, MoveTargetDto, PairingRef,
            RetakeBody, SessionBody,
        },
    },
};

type Shared_ = State<Arc<Shared>>;

pub(crate) fn router(shared: Arc<Shared>) -> Router {
    Router::new()
        .route("/phone", get(phone_page))
        .route("/phone/", get(phone_page))
        .route("/phone/icon.png", get(phone_icon))
        .route("/phone/manifest.webmanifest", get(manifest))
        .route("/assets/{*file}", get(asset))
        .route("/api/phone/v1/hello", get(hello))
        .route("/api/phone/v1/session", get(session))
        .route("/api/phone/v1/capture", post(capture))
        .route("/api/phone/v1/focus", post(focus))
        .route("/api/phone/v1/focus/clear", post(focus_clear))
        .route("/api/phone/v1/current", post(current))
        .route("/api/phone/v1/retake", post(retake))
        .route("/api/phone/v1/photos/{id}/move", post(move_photo))
        .route("/api/phone/v1/photos/{id}/delete", post(delete_photo))
        .route("/api/phone/v1/photos/{id}/preview", get(preview))
        .route("/api/phone/v1/containers/{id}/example", get(example))
        .route("/api/phone/v1/ws", get(crate::phone::ws::upgrade))
        .layer(middleware::from_fn(peers))
        .with_state(shared)
}

/// PH2: only this Mac and the private ranges of 10E MC-32 reach the server.
async fn peers(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = match addr.ip() {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(IpAddr::V6(v6), IpAddr::V4),
        ip => ip,
    };
    let nets = psm_infra::ftp::ReceiverConfig::default_allowed_peers();
    if ip.is_loopback() || psm_infra::ftp::peers::is_allowed(ip, &nets) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

/// The live pairing of a token (`last_seen_at` updated), else `pairing_unknown`.
pub(crate) fn verify(handles: &AppHandles, token: Option<&str>) -> Result<PhonePairing> {
    PhonePairings::new(handles.store.as_ref(), handles.clock.as_ref())
        .verify(token.unwrap_or_default())?
        .ok_or(Error::PairingUnknown)
}

/// The session guard (PH4): a POST acts only on the active session.
fn guard(handles: &AppHandles, session_id: i64) -> Result<()> {
    if handles.store.active_session_id()? == Some(session_id) {
        Ok(())
    } else {
        Err(Error::SessionChanged)
    }
}

fn status_of(error: &Error) -> StatusCode {
    StatusCode::from_u16(wire::error_code(error).0).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

fn log(route: &str, status: StatusCode, pairing: Option<i64>) {
    tracing::info!(
        target: "psm::phone",
        "phone.request route={route} status={} pairing={}",
        status.as_u16(),
        pairing.map_or("none".into(), |id| id.to_string())
    );
}

/// Runs `f` with the current handles and the request's pairing on a blocking thread; its value
/// is the JSON body, its error `{error: {code, message}}`.
async fn api<T, F>(
    shared: Arc<Shared>,
    route: &'static str,
    token: Option<String>,
    f: F,
) -> Response
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&AppHandles, &PhonePairing) -> Result<T> + Send + 'static,
{
    let joined =
        tokio::task::spawn_blocking(move || -> (Option<i64>, Result<serde_json::Value>) {
            let handles = match (shared.handles)() {
                Ok(handles) => handles,
                Err(e) => return (None, Err(e)),
            };
            let pairing = match verify(&handles, token.as_deref()) {
                Ok(pairing) => pairing,
                Err(e) => return (None, Err(e)),
            };
            crate::phone::ws::ensure_fed(&shared, &handles);
            let value = f(&handles, &pairing).and_then(|v| {
                serde_json::to_value(v).map_err(|e| Error::InvalidData(e.to_string()))
            });
            (Some(pairing.id), value)
        })
        .await
        .unwrap_or_else(|e| (None, Err(Error::InvalidData(e.to_string()))));
    match joined {
        (pairing, Ok(value)) => {
            log(route, StatusCode::OK, pairing);
            (StatusCode::OK, Json(value)).into_response()
        }
        (pairing, Err(error)) => {
            let status = status_of(&error);
            log(route, status, pairing);
            (status, Json(wire::error_body(&error))).into_response()
        }
    }
}

/// Runs `f` like [`api`]; its value is a file served with its content type.
async fn file<F>(shared: Arc<Shared>, route: &'static str, token: Option<String>, f: F) -> Response
where
    F: FnOnce(&AppHandles) -> Result<PathBuf> + Send + 'static,
{
    let joined =
        tokio::task::spawn_blocking(move || -> (Option<i64>, Result<(PathBuf, Vec<u8>)>) {
            let handles = match (shared.handles)() {
                Ok(handles) => handles,
                Err(e) => return (None, Err(e)),
            };
            let pairing = match verify(&handles, token.as_deref()) {
                Ok(pairing) => pairing,
                Err(e) => return (None, Err(e)),
            };
            crate::phone::ws::ensure_fed(&shared, &handles);
            let read = f(&handles).and_then(|path| {
                std::fs::read(&path)
                    .map(|bytes| (path.clone(), bytes))
                    .map_err(|_| Error::NotFound(path.to_string_lossy().into_owned()))
            });
            (Some(pairing.id), read)
        })
        .await
        .unwrap_or_else(|e| (None, Err(Error::InvalidData(e.to_string()))));
    match joined {
        (pairing, Ok((path, bytes))) => {
            log(route, StatusCode::OK, pairing);
            bytes_response(&path, bytes)
        }
        (pairing, Err(error)) => {
            let status = status_of(&error);
            log(route, status, pairing);
            (status, Json(wire::error_body(&error))).into_response()
        }
    }
}

fn bytes_response(path: &Path, bytes: Vec<u8>) -> Response {
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(wire::content_type(path)),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// A file under the web root (no token, PH3). A path that is not plain names below the root, or
/// that leaves it, is 404 (no directory traversal).
fn static_file(shared: &Shared, relative: &str) -> Response {
    let Some(root) = &shared.web_root else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let rel = Path::new(relative);
    if relative.is_empty() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let path = root.join(rel);
    let inside = match (path.canonicalize(), root.canonicalize()) {
        (Ok(file), Ok(root)) => file.starts_with(root) && file.is_file(),
        _ => false,
    };
    if !inside {
        return StatusCode::NOT_FOUND.into_response();
    }
    match std::fs::read(&path) {
        Ok(bytes) => bytes_response(&path, bytes),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn phone_page(State(shared): Shared_) -> Response {
    static_file(&shared, "phone.html")
}

async fn phone_icon(State(shared): Shared_) -> Response {
    static_file(&shared, "phone/icon.png")
}

async fn asset(State(shared): Shared_, UrlPath(file): UrlPath<String>) -> Response {
    static_file(&shared, &format!("assets/{file}"))
}

async fn manifest(State(shared): Shared_, headers: HeaderMap, RawQuery(q): RawQuery) -> Response {
    let token = token_of(&headers, q.as_deref());
    let joined = tokio::task::spawn_blocking({
        let token = token.clone();
        move || -> Result<i64> {
            let handles = (shared.handles)()?;
            Ok(verify(&handles, token.as_deref())?.id)
        }
    })
    .await
    .unwrap_or_else(|e| Err(Error::InvalidData(e.to_string())));
    match (joined, token) {
        (Ok(pairing), Some(token)) => {
            log("/phone/manifest.webmanifest", StatusCode::OK, Some(pairing));
            let mut response = Json(wire::manifest(&token)).into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/manifest+json"),
            );
            response
        }
        (Err(error), _) => {
            let status = status_of(&error);
            log("/phone/manifest.webmanifest", status, None);
            (status, Json(wire::error_body(&error))).into_response()
        }
        (Ok(_), None) => (
            StatusCode::UNAUTHORIZED,
            Json(wire::error_body(&Error::PairingUnknown)),
        )
            .into_response(),
    }
}

async fn hello(State(shared): Shared_, headers: HeaderMap, RawQuery(q): RawQuery) -> Response {
    let host = shared.host.clone();
    api(
        shared,
        "/api/phone/v1/hello",
        token_of(&headers, q.as_deref()),
        move |_, pairing| {
            Ok(Hello {
                api: 1,
                pairing: PairingRef {
                    id: pairing.id,
                    name: pairing.name.clone(),
                },
                mac_name: mac_name(host.as_ref()),
                features: FEATURES.to_vec(),
            })
        },
    )
    .await
}

fn default_shape(handles: &AppHandles) -> Shape {
    Shape::parse(&handles.settings.default_shape).unwrap_or(Shape::Square)
}

async fn session(State(shared): Shared_, headers: HeaderMap, RawQuery(q): RawQuery) -> Response {
    api(
        shared,
        "/api/phone/v1/session",
        token_of(&headers, q.as_deref()),
        |h, _| {
            phone_view(
                h.store.as_ref(),
                h.camera_runtime.current_shots().as_ref(),
                default_shape(h),
                h.store.active_session_id()?,
                h.clock.now_ms(),
            )
        },
    )
    .await
}

/// Parses a JSON body; a malformed one is `invalid_value`.
fn body<T: serde::de::DeserializeOwned>(bytes: &axum::body::Bytes) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| Error::InvalidValue(e.to_string()))
}

async fn capture(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    bytes: axum::body::Bytes,
) -> Response {
    api(
        shared,
        "/api/phone/v1/capture",
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: SessionBody = body(&bytes)?;
            guard(h, b.session_id)?;
            let id = CameraService::new(h.camera.as_ref(), h.health.as_ref()).capture()?;
            Ok(serde_json::json!({"capture_id": id.0}))
        },
    )
    .await
}

async fn focus(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    bytes: axum::body::Bytes,
) -> Response {
    api(
        shared,
        "/api/phone/v1/focus",
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: FocusBody = body(&bytes)?;
            guard(h, b.session_id)?;
            let id = live_view::focus(h.camera.as_ref(), FocusPoint { x: b.x, y: b.y })?;
            Ok(serde_json::json!({"focus_id": id.0}))
        },
    )
    .await
}

async fn focus_clear(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    bytes: axum::body::Bytes,
) -> Response {
    api(
        shared,
        "/api/phone/v1/focus/clear",
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: SessionBody = body(&bytes)?;
            guard(h, b.session_id)?;
            let id = live_view::focus_clear(h.camera.as_ref())?;
            Ok(serde_json::json!({"focus_id": id.0}))
        },
    )
    .await
}

fn hand(h: &AppHandles) -> HandAction<'_> {
    HandAction {
        store: h.store.as_ref(),
        clock: h.clock.as_ref(),
        events: h.events.as_ref(),
        history: h.history.as_ref(),
        shots: h.camera_runtime.current_shots(),
    }
}

const OK: fn() -> serde_json::Value = || serde_json::json!({"ok": true});

async fn current(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    bytes: axum::body::Bytes,
) -> Response {
    api(
        shared,
        "/api/phone/v1/current",
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: CurrentBody = body(&bytes)?;
            guard(h, b.session_id)?;
            h.camera_runtime.current_shots().set(
                h.store.as_ref(),
                h.events.as_ref(),
                b.session_id,
                b.container_id.map_or(Current::Free, Current::Shot),
                h.clock.now_ms(),
            )?;
            Ok(OK())
        },
    )
    .await
}

async fn retake(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    bytes: axum::body::Bytes,
) -> Response {
    api(
        shared,
        "/api/phone/v1/retake",
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: RetakeBody = body(&bytes)?;
            guard(h, b.session_id)?;
            template_placement::retake(&hand(h), b.session_id, b.container_id)?;
            Ok(OK())
        },
    )
    .await
}

async fn move_photo(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    UrlPath(photo_id): UrlPath<i64>,
    bytes: axum::body::Bytes,
) -> Response {
    let route = "/api/phone/v1/photos/{id}/move";
    api(
        shared,
        route,
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: MoveBody = body(&bytes)?;
            guard(h, b.session_id)?;
            phone_view::session_photo(h.store.as_ref(), Some(b.session_id), photo_id)?;
            let target = match b.target {
                MoveTargetDto::Container { container_id } => {
                    MoveTarget::Container(template_placement::container_final_no(
                        h.store.as_ref(),
                        b.session_id,
                        container_id,
                    )?)
                }
                MoveTargetDto::MainPosition { position } => MoveTarget::MainPosition(position),
                MoveTargetDto::ExtraEnd => MoveTarget::ExtraEnd,
                MoveTargetDto::Condition { position } => MoveTarget::Condition(position),
                MoveTargetDto::ConditionEnd => MoveTarget::ConditionEnd,
            };
            template_placement::move_to(&hand(h), photo_id, target)?;
            Ok(OK())
        },
    )
    .await
}

async fn delete_photo(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    UrlPath(photo_id): UrlPath<i64>,
    bytes: axum::body::Bytes,
) -> Response {
    let route = "/api/phone/v1/photos/{id}/delete";
    api(
        shared,
        route,
        token_of(&headers, q.as_deref()),
        move |h, _| {
            let b: SessionBody = body(&bytes)?;
            guard(h, b.session_id)?;
            phone_view::session_photo(h.store.as_ref(), Some(b.session_id), photo_id)?;
            ReviewService::new(h.store.as_ref(), h.clock.as_ref(), h.history.as_ref())
                .delete(photo_id)?;
            Ok(OK())
        },
    )
    .await
}

async fn preview(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    UrlPath(photo_id): UrlPath<i64>,
) -> Response {
    let route = "/api/phone/v1/photos/{id}/preview";
    file(shared, route, token_of(&headers, q.as_deref()), move |h| {
        let photo =
            phone_view::session_photo(h.store.as_ref(), h.store.active_session_id()?, photo_id)?;
        phone_view::preview_file(&photo)
            .map(PathBuf::from)
            .ok_or_else(|| Error::NotFound(format!("preview of photo {photo_id}")))
    })
    .await
}

async fn example(
    State(shared): Shared_,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
    UrlPath(container_id): UrlPath<i64>,
) -> Response {
    let route = "/api/phone/v1/containers/{id}/example";
    file(shared, route, token_of(&headers, q.as_deref()), move |h| {
        phone_view::example_file(
            h.store.as_ref(),
            &h.paths.root.join("templates"),
            h.store.active_session_id()?,
            container_id,
        )
    })
    .await
}
