//! Owner correction A12 (2026-11-18): the "Search this photo" window.
//!
//! One window per session (`search-<session>`) holding two webviews (Tauri's multiwebview):
//! - `search-ui-<session>`: the app's own page (trusted): the pinned photo, the browser controls,
//!   the saved pages and the selection bar with its actions;
//! - `search-web-<session>`: the web pages (untrusted), placed over the area the app's page
//!   leaves for it.
//!
//! Trust boundary (docs/guide/03 §A12): the page webview never reaches the app. Tauri refuses
//! every command from a remote origin (no capability names a remote URL), and
//! [`allowed_navigation`] keeps every frame of it on public `http`/`https` pages, so it can never
//! become an app-origin page. The selected text travels one way only: the app's page asks
//! ([`SELECTION_SCRIPT`], evaluated in the page) and checks the answer
//! ([`parse_selection_report`]). Writing a field or asking the model happens only from a click in
//! the app's own page.
//!
//! A14: every frame of the page webview also runs [`POPUP_SHIM`] at its start. It only turns a
//! page's "open in a new window" (links, forms, `window.open`) into a navigation of the page itself;
//! it reads and sends nothing, and the navigation check above still applies.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

/// The longest selected text the window takes (the same limit "Ask AI…" has).
pub const MAX_SELECTION_CHARS: usize = psm_research::browse::MAX_SELECTION_CHARS;
/// The longest raw answer the page may give to the selection question (bytes).
pub const MAX_REPORT_BYTES: usize = 32 * 1024;
const MAX_TITLE_CHARS: usize = 300;
const MAX_URL_CHARS: usize = 2_048;

/// Where a search starts: Google's own image search page, from which the photo is posted to
/// Google Lens (the route Google's page itself uses; probed 2026-11-18: `POST
/// https://lens.google.com/v3/upload` with the file field `encoded_image` answers `303` to the
/// results page `https://www.google.com/search?…&udm=26…`).
pub const GOOGLE_START: &str = "https://www.google.com/imghp?hl=en";
pub const LENS_UPLOAD: &str = "https://lens.google.com/v3/upload";

pub fn window_label(session_id: i64) -> String {
    format!("search-{session_id}")
}
pub fn ui_label(session_id: i64) -> String {
    format!("search-ui-{session_id}")
}
pub fn web_label(session_id: i64) -> String {
    format!("search-web-{session_id}")
}

/// The host of an `http(s)` URL, lowercased, without a trailing dot.
fn http_host(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        // IPv6 literal.
        format!("[{}]", stripped.split(']').next()?)
    } else {
        host_port.split(':').next()?.to_owned()
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    (!host.is_empty()).then_some((scheme, host))
}

/// A host the page webview may never load: this Mac, the local network's names and the app's
/// own origins (`tauri.localhost`, the dev server).
fn local_host(host: &str) -> bool {
    host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host == "0.0.0.0"
        || host.starts_with("127.")
        || host == "[::1]"
        || host == "[::]"
        || host.starts_with("[fe80:")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || (host.starts_with("172.")
            && host
                .split('.')
                .nth(1)
                .and_then(|n| n.parse::<u8>().ok())
                .is_some_and(|n| (16..=31).contains(&n)))
        || host.starts_with("169.254.")
        || !host.contains('.') && !host.starts_with('[')
}

/// SECURITY: whether a frame of the page webview may load `url`. Only public `http`/`https`
/// pages, and the blank documents pages make for their own frames. Never the app's own page
/// (`tauri:`, `asset:`, `ipc:`, the dev server on localhost), a file, or another scheme, so the
/// page webview can never turn into an app page that may call the app.
pub fn allowed_navigation(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    if lower == "about:blank" || lower == "about:srcdoc" {
        return true;
    }
    if let Some(inner) = lower.strip_prefix("blob:") {
        return http_host(inner).is_some_and(|(_, host)| !local_host(&host));
    }
    match http_host(url.trim()) {
        Some((_, host)) => !local_host(&host),
        None => false,
    }
}

/// Why the page webview may not load `url` (`None`: it may). For the logs: a word, never the
/// address itself.
pub fn refusal_reason(url: &str) -> Option<&'static str> {
    if allowed_navigation(url) {
        return None;
    }
    let lower = url.trim().to_ascii_lowercase();
    Some(if lower.is_empty() {
        "empty"
    } else if http_host(url.trim()).is_some()
        || lower
            .strip_prefix("blob:")
            .is_some_and(|inner| http_host(inner).is_some())
    {
        "local_address"
    } else {
        "not_a_web_page"
    })
}

/// A14: the script every frame of the page webview runs at its start (see `page_popups.js`): links
/// and pop-ups aimed at a new window open in this page instead. `go` navigates the top page (the
/// navigation check still applies).
pub const POPUP_SHIM: &str = include_str!("page_popups.js");
const POPUP_GO: &str =
    "function(u){try{window.top.location.href=u}catch(e){try{window.location.href=u}catch(e2){}}}";

pub fn popup_script() -> String {
    format!("({})({POPUP_GO});", POPUP_SHIM.trim())
}

/// What the page webview does when a page asks for a new window (A14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewWindow {
    /// Load the address in the page webview itself.
    OpenHere,
    /// Do nothing; the word says why (for the logs).
    Ignore(&'static str),
}

/// A14: a new window a page asks for opens in the page webview when it is a public web page. An
/// empty window (`about:blank`, which a page then points somewhere by script) is never loaded over
/// the page; the page script ([`POPUP_SHIM`]) sends such windows' address here itself.
pub fn new_window(url: &str) -> NewWindow {
    let lower = url.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.starts_with("about:") {
        return NewWindow::Ignore("blank_window");
    }
    match refusal_reason(url) {
        None if http_host(url.trim()).is_some() => NewWindow::OpenHere,
        None => NewWindow::Ignore("not_a_web_page"),
        Some(why) => NewWindow::Ignore(why),
    }
}

/// The script evaluated in the page to read what is selected. It only reads; its answer is a
/// string (JSON text) or `""`, whatever the page has changed around it.
pub const SELECTION_SCRIPT: &str = "(function(){try{\
var s=window.getSelection?(''+window.getSelection()):'';\
var o=JSON.stringify({text:(''+s).slice(0,2001),url:''+location.href,title:(''+document.title).slice(0,300)});\
return typeof o==='string'?o:'';}catch(e){return '';}})()";

/// What is selected on the page, as the app's page shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub text: String,
    pub url: String,
    pub title: String,
}

/// Why a selection answer was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejected {
    TooLarge,
    NotJson,
    UnexpectedShape,
    TooLong,
    NotAWebPage,
}

impl Rejected {
    pub fn sentence(&self) -> &'static str {
        match self {
            Rejected::TooLong => "The selection is too long — select at most 2,000 characters",
            Rejected::NotAWebPage => "This page cannot be used",
            _ => "The page gave an answer the app does not accept",
        }
    }
}

/// SECURITY: checks the page's answer to [`SELECTION_SCRIPT`] (the raw JSON the webview hands
/// back). Accepted: nothing selected (`None`), or exactly `{text, url, title}` as strings, the text
/// at most [`MAX_SELECTION_CHARS`] characters, the URL a page the webview may load. Anything else
/// is refused.
pub fn parse_selection_report(raw: &str) -> Result<Option<Selection>, Rejected> {
    if raw.len() > MAX_REPORT_BYTES {
        return Err(Rejected::TooLarge);
    }
    let raw = raw.trim();
    if raw.is_empty() || raw == "null" {
        return Ok(None);
    }
    // The webview hands back the script's string value as JSON text.
    let outer: serde_json::Value = serde_json::from_str(raw).map_err(|_| Rejected::NotJson)?;
    let inner = match outer {
        serde_json::Value::String(text) => text,
        _ => return Err(Rejected::UnexpectedShape),
    };
    if inner.is_empty() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(&inner).map_err(|_| Rejected::NotJson)?;
    let object = value.as_object().ok_or(Rejected::UnexpectedShape)?;
    if object.len() != 3
        || !["text", "url", "title"]
            .iter()
            .all(|k| object.contains_key(*k))
    {
        return Err(Rejected::UnexpectedShape);
    }
    let text = object["text"].as_str().ok_or(Rejected::UnexpectedShape)?;
    let url = object["url"].as_str().ok_or(Rejected::UnexpectedShape)?;
    let title = object["title"].as_str().ok_or(Rejected::UnexpectedShape)?;
    if url.chars().count() > MAX_URL_CHARS || !allowed_navigation(url) || url.starts_with("about:")
    {
        return Err(Rejected::NotAWebPage);
    }
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > MAX_SELECTION_CHARS {
        return Err(Rejected::TooLong);
    }
    Ok(Some(Selection {
        text: text.to_owned(),
        url: url.to_owned(),
        title: title.chars().take(MAX_TITLE_CHARS).collect(),
    }))
}

/// The browser identity the page webview gives (A13). WebKit's own default lacks Safari's
/// `Version/… Safari/…` part, and Google then serves its basic pages: the Lens upload lands on a
/// plain, empty search page instead of the visual matches (probed 2026-11-18: without these tokens
/// the redirect carries no `udm=26`; with them it does).
pub const PAGE_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15";

/// How many times the photo is posted to Google in one search (the first post, and one more when
/// Google lost it, e.g. on its cookie page). Never more: no loop.
pub const MAX_POSTS: u8 = 2;

/// What kind of page the page webview just loaded, for the Lens route and the logs (A13). The logs
/// get the host and this kind only: never the path's details, the query or the page's text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavKind {
    /// Google's cookie choice (`consent.google.*`).
    Consent,
    /// Google's "confirm you are a person" page.
    Sorry,
    /// Google's results page for a posted photo.
    Results,
    /// Google Lens's own address, in between.
    LensUpload,
    /// Any other Google page (the start page, the home page, an empty search page).
    GoogleOther,
    /// Any other site.
    Other,
}

impl NavKind {
    pub fn name(self) -> &'static str {
        match self {
            NavKind::Consent => "google_consent",
            NavKind::Sorry => "google_sorry",
            NavKind::Results => "google_results",
            NavKind::LensUpload => "lens_upload",
            NavKind::GoogleOther => "google_other",
            NavKind::Other => "other",
        }
    }
}

/// The host and kind of a loaded page (`None`: not an `http(s)` page).
pub fn nav_kind(url: &str) -> Option<(String, NavKind)> {
    let (_, host) = http_host(url)?;
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or("");
    let path_query = rest.find('/').map(|i| &rest[i..]).unwrap_or("/");
    let (path, query) = path_query.split_once('?').unwrap_or((path_query, ""));
    let query = query.split('#').next().unwrap_or("");
    let google =
        host == "google.com" || host.starts_with("www.google.") || host.starts_with("google.");
    let kind = if host.starts_with("consent.google.") {
        NavKind::Consent
    } else if host == "lens.google.com" {
        NavKind::LensUpload
    } else if google && path.starts_with("/sorry") {
        NavKind::Sorry
    } else if google
        && path.starts_with("/search")
        && query
            .split('&')
            .any(|pair| pair.starts_with("vsrid=") || pair == "udm=26" || pair.starts_with("lns_"))
    {
        NavKind::Results
    } else if google {
        NavKind::GoogleOther
    } else {
        NavKind::Other
    };
    Some((host, kind))
}

/// Where the Google Lens route of one window stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum LensPhase {
    /// Nothing asked, or the person is browsing.
    Idle,
    /// Google's start page is loading; the photo is posted once it has.
    Opening,
    /// Google shows its cookie choice; the photo is posted once the person has chosen.
    Consent,
    /// The photo was posted; waiting for the results page.
    Uploading,
    /// Google's results page for the photo.
    Results { url: String },
    /// The route did not work (the window shows its own list instead).
    Failed { why: String },
}

const WHY_PERSON: &str = "Google asks to confirm you are a person";
const WHY_NOT_OPENED: &str = "Google's page did not open";
const WHY_NO_RESULTS: &str = "Google showed a page without results for the photo";

/// What a finished page load means for the Lens route, given how often the photo was posted
/// already: the next phase, and whether the photo is to be posted now (on the page just loaded).
///
/// A13: the photo is posted again (up to [`MAX_POSTS`] in all) whenever the route, still waiting,
/// lands on a Google page that is not the results: after the cookie choice, or when Google answers
/// the post with its home page. After that the route gives up and the window shows its own list.
pub fn lens_on_page(phase: &LensPhase, posts: u8, url: &str) -> (LensPhase, bool) {
    let waiting = matches!(
        phase,
        LensPhase::Opening | LensPhase::Consent | LensPhase::Uploading
    );
    if !waiting {
        return (phase.clone(), false);
    }
    let Some((_, kind)) = nav_kind(url) else {
        return (phase.clone(), false);
    };
    let failed = |why: &str| (LensPhase::Failed { why: why.into() }, false);
    let post_again = || {
        if posts < MAX_POSTS {
            (LensPhase::Uploading, true)
        } else {
            failed(WHY_NO_RESULTS)
        }
    };
    match kind {
        NavKind::Consent => (LensPhase::Consent, false),
        NavKind::Sorry => failed(WHY_PERSON),
        NavKind::Results if *phase == LensPhase::Uploading => (
            LensPhase::Results {
                url: url.to_owned(),
            },
            false,
        ),
        // Results without our post (an old address Google went back to): post the photo.
        NavKind::Results | NavKind::GoogleOther => post_again(),
        NavKind::LensUpload => (phase.clone(), false),
        NavKind::Other if *phase == LensPhase::Uploading => failed(WHY_NO_RESULTS),
        NavKind::Other => failed(WHY_NOT_OPENED),
    }
}

/// The script that posts the photo to Google Lens from Google's own page (a same-site form post,
/// as Google's page does it when a picture is dropped on it). `base64` is the JPEG.
pub fn upload_script(base64: &str) -> String {
    format!(
        "(function(b){{try{{var s=atob(b);var u=new Uint8Array(s.length);\
for(var i=0;i<s.length;i++)u[i]=s.charCodeAt(i);\
var f=new File([u],'photo.jpg',{{type:'image/jpeg'}});var d=new DataTransfer();d.items.add(f);\
var form=document.createElement('form');form.method='POST';form.enctype='multipart/form-data';\
form.action='{LENS_UPLOAD}?hl=en&st='+Date.now();\
var i2=document.createElement('input');i2.type='file';i2.name='encoded_image';i2.files=d.files;\
form.appendChild(i2);form.style.display='none';document.documentElement.appendChild(form);\
HTMLFormElement.prototype.submit.call(form);}}catch(e){{}}}})('{base64}')"
    )
}

/// One window's Lens route and the photo it posts.
#[derive(Debug, Clone)]
pub struct LensState {
    pub phase: LensPhase,
    /// The JPEG to post, base64, kept until the route is over (results or given up), so it can be
    /// posted again.
    pub photo: Option<String>,
    pub photo_id: i64,
    /// How often the photo was posted in this search.
    pub posts: u8,
}

/// A rectangle in logical points (CSS pixels), from the window's content's top-left.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The tallest title bar the layout accepts (anything above is a misreading and is ignored).
pub const MAX_TITLE_INSET: f64 = 120.0;

/// A13: the app's page fills the window's content area below the title bar. On macOS the window's
/// content view runs under the title bar (probed 2026-11-18: content 900 pt tall, the area below
/// the title bar 868 pt), so a webview placed at the top would hide its first 32 pt behind it.
/// `inner_*` is the content view's size and `title_inset` the title bar's height, in points.
pub fn ui_bounds(inner_width: f64, inner_height: f64, title_inset: f64) -> Bounds {
    let inset = if title_inset.is_finite() && (0.0..=MAX_TITLE_INSET).contains(&title_inset) {
        title_inset
    } else {
        0.0
    };
    Bounds {
        x: 0.0,
        y: inset,
        width: inner_width.max(1.0),
        height: (inner_height - inset).max(1.0),
    }
}

/// A13: the page webview's place: the browser area as the app's page measured it (CSS pixels of
/// the app's page, which are logical points, never device pixels), moved by where the app's page
/// sits in the window, and kept inside the app's page.
pub fn web_bounds(area: Bounds, ui: Bounds) -> Bounds {
    let finite = |v: f64| if v.is_finite() { v } else { 0.0 };
    let x = finite(area.x).clamp(0.0, ui.width);
    let y = finite(area.y).clamp(0.0, ui.height);
    Bounds {
        x: ui.x + x,
        y: ui.y + y,
        width: finite(area.width).clamp(1.0, (ui.width - x).max(1.0)),
        height: finite(area.height).clamp(1.0, (ui.height - y).max(1.0)),
    }
}

/// The windows' Lens routes and layouts, by session (managed by Tauri).
#[derive(Default, Clone)]
pub struct SearchWindows {
    pub lens: Arc<Mutex<HashMap<i64, LensState>>>,
    /// Where each window's app page sits (see [`ui_bounds`]).
    pub layout: Arc<Mutex<HashMap<i64, Bounds>>>,
}

impl SearchWindows {
    /// A page finished loading in `session`'s page webview: the next phase (when it changed or the
    /// photo is posted), and the script to run in it (the photo post) when it is time.
    pub fn page_loaded(&self, session_id: i64, url: &str) -> (Option<LensPhase>, Option<String>) {
        let mut lens = self.lens.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = lens.get_mut(&session_id) else {
            return (None, None);
        };
        let before = state.phase.clone();
        let (next, post) = lens_on_page(&state.phase, state.posts, url);
        let script = if post {
            state.photo.as_deref().map(upload_script)
        } else {
            None
        };
        // Time to post but no photo to post: the route is over.
        let next = if post && script.is_none() {
            LensPhase::Idle
        } else {
            next
        };
        if script.is_some() {
            state.posts += 1;
        }
        if matches!(
            next,
            LensPhase::Results { .. } | LensPhase::Failed { .. } | LensPhase::Idle
        ) {
            state.photo = None;
        }
        state.phase = next.clone();
        ((next != before || script.is_some()).then_some(next), script)
    }

    pub fn start(&self, session_id: i64, photo_id: i64, jpeg_base64: String) {
        self.lens.lock().unwrap_or_else(|e| e.into_inner()).insert(
            session_id,
            LensState {
                phase: LensPhase::Opening,
                photo: Some(jpeg_base64),
                photo_id,
                posts: 0,
            },
        );
    }

    pub fn phase(&self, session_id: i64) -> Option<LensPhase> {
        self.lens
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&session_id)
            .map(|s| s.phase.clone())
    }

    /// How often the photo was posted in the current search.
    pub fn posts(&self, session_id: i64) -> u8 {
        self.lens
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&session_id)
            .map_or(0, |s| s.posts)
    }

    pub fn fail(&self, session_id: i64, why: &str) {
        if let Some(state) = self
            .lens
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&session_id)
        {
            state.photo = None;
            state.phase = LensPhase::Failed { why: why.into() };
        }
    }

    pub fn set_layout(&self, session_id: i64, ui: Bounds) {
        self.layout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(session_id, ui);
    }

    /// Where the app's page sits (the whole content area until the window has laid it out).
    pub fn layout(&self, session_id: i64) -> Option<Bounds> {
        self.layout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&session_id)
            .copied()
    }

    pub fn forget(&self, session_id: i64) {
        self.lens
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&session_id);
        self.layout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&session_id);
    }
}

/// The page webview's page, for the app's page (event `search_web_page`).
#[derive(Debug, Clone, Serialize)]
pub struct PagePayload {
    pub session_id: i64,
    pub url: String,
    pub title: Option<String>,
    pub loading: bool,
}

/// The Lens route's phase, for the app's page (event `search_lens`).
#[derive(Debug, Clone, Serialize)]
pub struct LensPayload {
    pub session_id: i64,
    #[serde(flatten)]
    pub phase: LensPhase,
}

#[cfg(desktop)]
pub mod window {
    //! Creating the window and its two webviews (macOS; not reachable from tests).

    use super::*;
    use tauri::{
        AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl,
        webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    };

    /// Opens the session's search window on `photo_id`, or brings it forward and switches its
    /// pinned photo when it is open.
    pub fn open<R: Runtime>(
        app: &AppHandle<R>,
        session_id: i64,
        photo_id: i64,
        title: &str,
    ) -> tauri::Result<()> {
        if let Some(window) = app.get_window(&window_label(session_id)) {
            let _ = window.unminimize();
            window.set_focus()?;
            app.emit(
                "search_photo",
                serde_json::json!({"session_id": session_id, "photo_id": photo_id}),
            )?;
            return Ok(());
        }
        let width = 1_440.0;
        let height = 900.0;
        let window = tauri::window::WindowBuilder::new(app, window_label(session_id))
            .title(title)
            .inner_size(width, height)
            .min_inner_size(1_000.0, 640.0)
            .build()?;
        // A13: no `auto_resize` (Tauri scales it by the window's size, title bar included); the
        // app's page is laid out below the title bar here and on every resize.
        let ui = WebviewBuilder::new(
            ui_label(session_id),
            WebviewUrl::App(format!("index.html#/search/{session_id}/{photo_id}").into()),
        );
        window.add_child(
            ui,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, height),
        )?;
        let page = page_webview(app, session_id);
        // Hidden until the app's page says where the browser area is.
        let web = window.add_child(
            page,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1.0, 1.0),
        )?;
        let _ = web.hide();
        let laid_out = window.clone();
        window.run_on_main_thread(move || lay_out(&laid_out, session_id))?;
        let handle = app.clone();
        let resized = window.clone();
        window.on_window_event(move |event| match event {
            tauri::WindowEvent::Destroyed => handle.state::<SearchWindows>().forget(session_id),
            tauri::WindowEvent::Resized(_) | tauri::WindowEvent::ScaleFactorChanged { .. } => {
                lay_out(&resized, session_id)
            }
            _ => {}
        });
        tracing::info!(
            target: "psm::search",
            "search_window.opened session={session_id} photo={photo_id}"
        );
        Ok(())
    }

    /// The content view's size and the title bar's height, in points (macOS, main thread only).
    #[cfg(target_os = "macos")]
    fn content_and_inset<R: Runtime>(window: &tauri::Window<R>) -> Option<(f64, f64, f64)> {
        use objc2::{msg_send, runtime::AnyObject};
        use objc2_core_foundation::CGRect;
        let ns_window = window.ns_window().ok()? as *mut AnyObject;
        if ns_window.is_null() {
            return None;
        }
        // SAFETY: called on the main thread (window events and `run_on_main_thread`), on the
        // window's own NSWindow; `contentView` is checked for nil before use.
        unsafe {
            let content: *mut AnyObject = msg_send![ns_window, contentView];
            if content.is_null() {
                return None;
            }
            let frame: CGRect = msg_send![content, frame];
            // In window coordinates (origin bottom-left): the area below the title bar.
            let layout: CGRect = msg_send![ns_window, contentLayoutRect];
            let inset = frame.size.height - (layout.origin.y + layout.size.height);
            Some((frame.size.width, frame.size.height, inset))
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn content_and_inset<R: Runtime>(window: &tauri::Window<R>) -> Option<(f64, f64, f64)> {
        let scale = window.scale_factor().ok()?;
        let size = window.inner_size().ok()?.to_logical::<f64>(scale);
        Some((size.width, size.height, 0.0))
    }

    /// A13: puts the app's page below the title bar, filling the rest of the window, and keeps
    /// its place for the page webview's (main thread).
    fn lay_out<R: Runtime>(window: &tauri::Window<R>, session_id: i64) {
        let Some((width, height, inset)) = content_and_inset(window) else {
            return;
        };
        let ui = ui_bounds(width, height, inset);
        let app = window.app_handle();
        app.state::<SearchWindows>().set_layout(session_id, ui);
        if let Some(page) = app.get_webview(&ui_label(session_id)) {
            let _ = page.set_position(LogicalPosition::new(ui.x, ui.y));
            let _ = page.set_size(LogicalSize::new(ui.width, ui.height));
        }
        tracing::debug!(
            target: "psm::search",
            "search_window.layout session={session_id} content={width:.0}x{height:.0} title_bar={inset:.0}"
        );
    }

    fn page_webview<R: Runtime>(app: &AppHandle<R>, session_id: i64) -> WebviewBuilder<R> {
        let blank = "about:blank".parse().expect("about:blank");
        let titles = app.clone();
        let loads = app.clone();
        let popups = app.clone();
        WebviewBuilder::new(web_label(session_id), WebviewUrl::External(blank))
            // A13: Safari's identity, so Google serves its current pages and Lens results.
            .user_agent(PAGE_USER_AGENT)
            // A14: links and pop-ups aimed at a new window open in this page (every frame).
            .initialization_script_for_all_frames(popup_script())
            .on_navigation(move |url| {
                let refused = refusal_reason(url.as_str());
                if let Some(why) = refused {
                    tracing::info!(
                        target: "psm::search",
                        "search_window.navigation_refused session={session_id} scheme={} host={} why={why}",
                        url.scheme(),
                        url.host_str().unwrap_or("-")
                    );
                }
                refused.is_none()
            })
            .on_download(|_, _| false)
            .on_new_window(move |url, _| {
                // A link that would open a new window opens here instead.
                let host = url.host_str().unwrap_or("-").to_owned();
                match new_window(url.as_str()) {
                    NewWindow::OpenHere => {
                        if let Some(web) = popups.get_webview(&web_label(session_id)) {
                            let _ = web.navigate(url);
                        }
                        tracing::info!(
                            target: "psm::search",
                            "search_window.new_window session={session_id} host={host} result=opened_here"
                        );
                    }
                    NewWindow::Ignore(why) => tracing::info!(
                        target: "psm::search",
                        "search_window.new_window session={session_id} host={host} result=ignored why={why}"
                    ),
                }
                NewWindowResponse::Deny
            })
            .on_document_title_changed(move |webview, title| {
                let url = webview.url().map(|u| u.to_string()).unwrap_or_default();
                let _ = titles.emit(
                    "search_web_page",
                    PagePayload {
                        session_id,
                        url,
                        title: Some(title.chars().take(MAX_TITLE_CHARS).collect()),
                        loading: false,
                    },
                );
            })
            .on_page_load(move |webview, payload| {
                let url = payload.url().to_string();
                let loading = matches!(payload.event(), PageLoadEvent::Started);
                let _ = loads.emit(
                    "search_web_page",
                    PagePayload {
                        session_id,
                        url: url.clone(),
                        title: None,
                        loading,
                    },
                );
                if loading {
                    return;
                }
                // A13: every loaded page's host and kind (never its path details, query or text).
                if let Some((host, kind)) = nav_kind(&url) {
                    tracing::info!(
                        target: "psm::search",
                        "search_window.navigation session={session_id} host={host} kind={}",
                        kind.name()
                    );
                }
                let windows = loads.state::<SearchWindows>();
                let (phase, script) = windows.page_loaded(session_id, &url);
                if let Some(script) = script {
                    let _ = webview.eval(script);
                    let attempt = windows.posts(session_id);
                    let step = if attempt > 1 { "photo_posted_again" } else { "photo_posted" };
                    tracing::info!(
                        target: "psm::search",
                        "search_window.lens_route step={step} attempt={attempt} session={session_id}"
                    );
                }
                if let Some(phase) = phase {
                    log_phase(session_id, &phase);
                    let _ = loads.emit("search_lens", LensPayload { session_id, phase });
                }
            })
    }

    fn log_phase(session_id: i64, phase: &LensPhase) {
        let (state, why) = match phase {
            LensPhase::Idle => ("idle", ""),
            LensPhase::Opening => ("opening", ""),
            LensPhase::Consent => ("consent", ""),
            LensPhase::Uploading => ("uploading", ""),
            LensPhase::Results { .. } => ("ok", ""),
            LensPhase::Failed { why } => ("fallback", why.as_str()),
        };
        tracing::info!(
            target: "psm::search",
            "search_window.lens_route result={state} session={session_id} why={why:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a12_page_webview_loads_only_public_web_pages() {
        for ok in [
            "https://www.google.com/search?q=teak",
            "http://example.com/a",
            "https://shop.example.se:8443/x?y#z",
            "about:blank",
            "about:srcdoc",
            "blob:https://www.google.com/1234-5678",
        ] {
            assert!(allowed_navigation(ok), "{ok}");
        }
        for refused in [
            // The app's own origins: a page that loaded these could call the app.
            "tauri://localhost/index.html",
            "https://tauri.localhost/index.html",
            "http://tauri.localhost/",
            "http://localhost:1420/index.html#/settings",
            "http://127.0.0.1:1420/",
            "http://[::1]:1420/",
            "asset://localhost/%2FUsers%2Fme%2Fphoto.jpg",
            "ipc://localhost/research_ask",
            "file:///Users/me/Library/Application%20Support/x.db",
            "javascript:alert(1)",
            "data:text/html,<script>1</script>",
            "blob:tauri://localhost/123",
            "blob:http://localhost:1420/123",
            "mailto:a@b.example",
            "http://192.168.1.10/router",
            "http://10.0.0.1/",
            "http://172.16.0.1/",
            "http://169.254.169.254/latest",
            "http://printer.local/",
            "http://intranet/",
            "",
        ] {
            assert!(!allowed_navigation(refused), "{refused}");
        }
    }

    fn report(value: serde_json::Value) -> String {
        serde_json::Value::String(value.to_string()).to_string()
    }

    #[test]
    fn a12_selection_answer_is_checked_before_the_app_shows_it() {
        let ok = report(serde_json::json!({
            "text": "  designed by Kai Kristiansen ",
            "url": "https://e1.example/p",
            "title": "Teak sideboard"
        }));
        assert_eq!(
            parse_selection_report(&ok).unwrap(),
            Some(Selection {
                text: "designed by Kai Kristiansen".into(),
                url: "https://e1.example/p".into(),
                title: "Teak sideboard".into(),
            })
        );
        // Nothing selected.
        assert_eq!(parse_selection_report("").unwrap(), None);
        assert_eq!(parse_selection_report("\"\"").unwrap(), None);
        let blank =
            report(serde_json::json!({"text": "   ", "url": "https://e1.example/", "title": ""}));
        assert_eq!(parse_selection_report(&blank).unwrap(), None);
        // Too long a selection.
        let long = report(serde_json::json!({
            "text": "x".repeat(MAX_SELECTION_CHARS + 1),
            "url": "https://e1.example/",
            "title": ""
        }));
        assert_eq!(parse_selection_report(&long), Err(Rejected::TooLong));
        // An oversize answer is refused before it is read.
        let huge = format!("\"{}\"", "y".repeat(MAX_REPORT_BYTES + 1));
        assert_eq!(parse_selection_report(&huge), Err(Rejected::TooLarge));
        // Unexpected shapes: extra keys, missing keys, wrong types, not a string, not JSON.
        for bad in [
            report(
                serde_json::json!({"text": "a", "url": "https://e1.example/", "title": "", "action": "put"}),
            ),
            report(serde_json::json!({"text": "a", "url": "https://e1.example/"})),
            report(serde_json::json!({"text": 1, "url": "https://e1.example/", "title": ""})),
            report(serde_json::json!(["a"])),
            serde_json::json!({"text": "a", "url": "https://e1.example/", "title": ""}).to_string(),
            "not json".to_owned(),
            "\"not json inside\"".to_owned(),
        ] {
            assert!(parse_selection_report(&bad).is_err(), "{bad}");
        }
        // A page that is not a public web page.
        for url in [
            "tauri://localhost/",
            "http://localhost:1420/",
            "about:blank",
        ] {
            let bad = report(serde_json::json!({"text": "a", "url": url, "title": ""}));
            assert_eq!(
                parse_selection_report(&bad),
                Err(Rejected::NotAWebPage),
                "{url}"
            );
        }
    }

    #[test]
    fn a12_lens_route_posts_once_on_googles_page_and_finds_the_results() {
        let windows = SearchWindows::default();
        windows.start(5, 9, "QUJD".into());
        // Consent first (EU): no post on the consent page.
        let (phase, script) = windows.page_loaded(5, "https://consent.google.com/m?continue=x");
        assert_eq!(phase, Some(LensPhase::Consent));
        assert!(script.is_none());
        // Back on Google after the choice: the photo is posted, once.
        let (phase, script) = windows.page_loaded(5, "https://www.google.com/imghp?hl=en");
        assert_eq!(phase, Some(LensPhase::Uploading));
        let script = script.unwrap();
        assert!(script.contains("lens.google.com/v3/upload"));
        assert!(script.contains("encoded_image"));
        assert!(script.contains("'QUJD'"));
        // The results page.
        let results = "https://www.google.com/search?vsrid=abc&udm=26&lns_mode=un";
        let (phase, script) = windows.page_loaded(5, results);
        assert_eq!(
            phase,
            Some(LensPhase::Results {
                url: results.into()
            })
        );
        assert!(script.is_none());
        // Browsing on: nothing changes, nothing is posted again.
        assert_eq!(windows.page_loaded(5, "https://e1.example/p"), (None, None));
        assert_eq!(
            windows.phase(5),
            Some(LensPhase::Results {
                url: results.into()
            })
        );
        // Another session's window has no route.
        assert_eq!(
            windows.page_loaded(6, "https://www.google.com/"),
            (None, None)
        );
    }

    #[test]
    fn a12_lens_route_falls_back_when_google_does_not_answer_with_results() {
        let (phase, post) = lens_on_page(&LensPhase::Opening, 0, "https://example.com/");
        assert!(matches!(phase, LensPhase::Failed { .. }));
        assert!(!post);
        let (phase, _) = lens_on_page(
            &LensPhase::Uploading,
            1,
            "https://www.google.com/sorry/index",
        );
        assert!(matches!(phase, LensPhase::Failed { .. }));
        // A13: Google's home page after the last allowed post: the route gives up.
        let (phase, post) = lens_on_page(
            &LensPhase::Uploading,
            MAX_POSTS,
            "https://www.google.com/imghp",
        );
        assert!(matches!(phase, LensPhase::Failed { .. }));
        assert!(!post);
        // Lens's own address in between is fine.
        let (phase, _) = lens_on_page(
            &LensPhase::Uploading,
            1,
            "https://lens.google.com/v3/upload?x",
        );
        assert_eq!(phase, LensPhase::Uploading);
    }

    /// Runs a sequence of page loads through one window's route: the phases and the posts made.
    fn walk(urls: &[&str]) -> (Vec<LensPhase>, usize, SearchWindows) {
        let windows = SearchWindows::default();
        windows.start(1, 2, "QUJD".into());
        let mut posts = 0;
        let mut phases = Vec::new();
        for url in urls {
            let (_, script) = windows.page_loaded(1, url);
            posts += usize::from(script.is_some());
            phases.push(windows.phase(1).unwrap());
        }
        (phases, posts, windows)
    }

    const START: &str = "https://www.google.com/imghp?hl=en";
    const CONSENT: &str = "https://consent.google.com/ml?continue=https://www.google.com/imghp";
    const HOME: &str = "https://www.google.com/?hl=en";
    const UPLOAD: &str = "https://lens.google.com/v3/upload?hl=en&st=1";
    const RESULTS: &str = "https://www.google.com/search?vsrid=x&udm=26&lns_mode=un&hl=en";

    #[test]
    fn a13_the_photo_is_posted_again_when_google_lost_it_after_the_cookie_choice() {
        // The post went to the cookie page, which then went to Google's home page.
        let (phases, posts, _) = walk(&[START, CONSENT, HOME, UPLOAD, RESULTS]);
        assert_eq!(posts, 2);
        assert_eq!(
            phases.last(),
            Some(&LensPhase::Results {
                url: RESULTS.into()
            })
        );
        let (phases, posts, _) = walk(&[START, UPLOAD, CONSENT, HOME, UPLOAD, RESULTS]);
        assert_eq!(posts, 2);
        assert!(matches!(phases.last(), Some(LensPhase::Results { .. })));
        // Cookie choice before any post: one post is enough.
        let (_, posts, _) = walk(&[CONSENT, START, UPLOAD, RESULTS]);
        assert_eq!(posts, 1);
    }

    #[test]
    fn a13_the_photo_is_posted_at_most_twice_then_the_window_shows_its_own_list() {
        let (phases, posts, windows) = walk(&[START, HOME, HOME, HOME, HOME]);
        assert_eq!(posts, usize::from(MAX_POSTS));
        assert!(matches!(phases.last(), Some(LensPhase::Failed { .. })));
        // Over: the photo is dropped and nothing is posted later on any Google page.
        assert_eq!(windows.page_loaded(1, START), (None, None));
        assert_eq!(windows.posts(1), MAX_POSTS);
        // A plain search page without the photo's results counts as "no results".
        let (phases, posts, _) = walk(&[START, "https://www.google.com/search?hl=en", HOME]);
        assert_eq!(posts, 2);
        assert!(matches!(phases.last(), Some(LensPhase::Failed { .. })));
    }

    #[test]
    fn a13_navigations_are_named_by_host_and_kind_only() {
        let kind = |u: &str| nav_kind(u).map(|(h, k)| (h, k.name()));
        assert_eq!(
            kind(RESULTS),
            Some(("www.google.com".into(), "google_results"))
        );
        assert_eq!(
            kind("https://www.google.com/search?q=teak+chair"),
            Some(("www.google.com".into(), "google_other"))
        );
        assert_eq!(
            kind(CONSENT),
            Some(("consent.google.com".into(), "google_consent"))
        );
        assert_eq!(
            kind(UPLOAD),
            Some(("lens.google.com".into(), "lens_upload"))
        );
        assert_eq!(
            kind("https://www.google.se/sorry/index?continue=x"),
            Some(("www.google.se".into(), "google_sorry"))
        );
        assert_eq!(
            kind("https://shop.example.se/products/teak?x=1"),
            Some(("shop.example.se".into(), "other"))
        );
        assert_eq!(kind("about:blank"), None);
    }

    #[test]
    fn a14_a_new_window_a_page_asks_for_opens_in_the_page_itself() {
        for url in [
            "https://www.rusta.com/sv/vattenflaska-500-ml",
            "https://www.google.com/url?q=https://www.amazon.se/x&sa=U",
            "http://shop.example.se/p?id=1#top",
        ] {
            assert_eq!(new_window(url), NewWindow::OpenHere, "{url}");
        }
        // An empty window is never loaded over the page (the page script sends its address).
        assert_eq!(new_window("about:blank"), NewWindow::Ignore("blank_window"));
        assert_eq!(new_window(""), NewWindow::Ignore("blank_window"));
        // Never anything but a public web page.
        assert_eq!(
            new_window("http://localhost:1420/index.html"),
            NewWindow::Ignore("local_address")
        );
        assert_eq!(
            new_window("tauri://localhost/"),
            NewWindow::Ignore("not_a_web_page")
        );
        assert_eq!(
            new_window("javascript:alert(1)"),
            NewWindow::Ignore("not_a_web_page")
        );
        assert_eq!(
            new_window("blob:https://www.google.com/1"),
            NewWindow::Ignore("not_a_web_page")
        );
    }

    #[test]
    fn a14_refused_navigations_say_why_in_a_word() {
        assert_eq!(refusal_reason("https://www.google.com/search?q=x"), None);
        assert_eq!(refusal_reason("about:blank"), None);
        assert_eq!(refusal_reason(""), Some("empty"));
        assert_eq!(
            refusal_reason("http://192.168.1.10/router"),
            Some("local_address")
        );
        assert_eq!(
            refusal_reason("blob:http://localhost:1420/1"),
            Some("local_address")
        );
        assert_eq!(refusal_reason("file:///etc/hosts"), Some("not_a_web_page"));
        assert_eq!(
            refusal_reason("intent://x#Intent;end"),
            Some("not_a_web_page")
        );
    }

    #[test]
    fn a14_the_page_script_turns_new_windows_into_this_page_and_reaches_nothing_else() {
        let script = popup_script();
        // The script file is wrapped as `(<function>)(<go>);`.
        assert!(script.starts_with('(') && script.ends_with(");"));
        assert!(POPUP_SHIM.contains("window.open = function"));
        assert!(POPUP_SHIM.contains("addEventListener('click', retarget, true)"));
        assert!(script.contains("window.top.location.href=u"));
        // It never talks to the app.
        for word in [
            "__TAURI",
            "ipc",
            "invoke",
            "postMessage(",
            "fetch(",
            "XMLHttpRequest",
        ] {
            assert!(!POPUP_SHIM.contains(word), "{word}");
        }
    }

    #[test]
    fn a13_page_identity_is_safari_so_google_serves_its_current_pages() {
        // Google serves its basic pages (no Lens results) to WebKit without these tokens.
        assert!(PAGE_USER_AGENT.contains("AppleWebKit/"));
        assert!(PAGE_USER_AGENT.contains(" Version/"));
        assert!(PAGE_USER_AGENT.ends_with(" Safari/605.1.15"));
        assert!(!PAGE_USER_AGENT.contains('\n') && !PAGE_USER_AGENT.contains("  "));
    }

    #[test]
    fn a13_the_app_page_sits_below_the_title_bar_and_the_browser_in_its_area() {
        // Probed on the dev Mac: content view 1440x900 pt, 32 pt of it under the title bar.
        let ui = ui_bounds(1440.0, 900.0, 32.0);
        assert_eq!(
            ui,
            Bounds {
                x: 0.0,
                y: 32.0,
                width: 1440.0,
                height: 868.0
            }
        );
        // The browser area as the app's page measures it (CSS pixels = points, at any scale):
        // right of the photo panel, under the toolbar, above the selection bar.
        let area = Bounds {
            x: 381.0,
            y: 41.0,
            width: 1059.0,
            height: 767.0,
        };
        assert_eq!(
            web_bounds(area, ui),
            Bounds {
                x: 381.0,
                y: 73.0,
                width: 1059.0,
                height: 767.0
            }
        );
        // Never outside the app's page, whatever the page reports.
        let wild = Bounds {
            x: -50.0,
            y: 5000.0,
            width: 9000.0,
            height: f64::NAN,
        };
        let placed = web_bounds(wild, ui);
        assert_eq!((placed.x, placed.y), (0.0, 32.0 + 868.0));
        assert_eq!((placed.width, placed.height), (1440.0, 1.0));
        // A misread title bar is ignored rather than pushing the page off screen.
        assert_eq!(ui_bounds(1440.0, 900.0, 500.0).y, 0.0);
        assert_eq!(ui_bounds(1440.0, 900.0, f64::NAN).y, 0.0);
        assert_eq!(ui_bounds(1440.0, 900.0, -3.0).y, 0.0);
    }
}
