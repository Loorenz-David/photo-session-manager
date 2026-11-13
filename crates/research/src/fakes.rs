//! Stand-in providers for tests (`master_plan_research.md` §7.5): replies scripted per stage and
//! unit, a log of every request received (each image by SHA-256 and long edge), a queue of
//! `CallFailure` faults, and a `before_call` hook run when a call arrives, before the reply.
//!
//! The hook runs with no lock held, so it may read the database, change settings, or stop the run
//! (a test "kill" unwinds out of the hook with `std::panic::resume_unwind`).

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use serde_json::{Value as Json, json};

use crate::{
    domain::{CallFailure, FailureKind, Stage},
    images::ResearchImage,
    ports::{
        Exchange, ModelProvider, ModelReply, ModelRequest, PageReader, PageReply, SearchCandidate,
        SearchReply, VisualSearchProvider,
    },
};

/// One request a fake received.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeCall {
    /// `model`, `search` or `reader`.
    pub port: &'static str,
    pub stage: Option<Stage>,
    pub unit_key: String,
    /// Each image as (SHA-256, long edge).
    pub images: Vec<(String, u32)>,
    pub listing_images: Vec<String>,
    pub input: Json,
    pub language: String,
    pub model: String,
    pub url: Option<String>,
}

pub type Hook = Box<dyn FnMut(&FakeCall) + Send>;

struct Script<R> {
    replies: HashMap<String, R>,
    faults: VecDeque<(Option<String>, CallFailure)>,
    log: Vec<FakeCall>,
}

impl<R> Default for Script<R> {
    fn default() -> Self {
        Self {
            replies: HashMap::new(),
            faults: VecDeque::new(),
            log: Vec::new(),
        }
    }
}

/// The parts every fake shares.
struct Core<R> {
    script: Mutex<Script<R>>,
    hook: Arc<Mutex<Option<Hook>>>,
}

impl<R: Clone> Core<R> {
    fn new() -> Self {
        Self {
            script: Mutex::new(Script::default()),
            hook: Arc::new(Mutex::new(None)),
        }
    }

    /// Logs the call, runs the hook (no lock held), then hands back the first queued fault that
    /// applies to `key`, or the scripted reply.
    fn arrive(&self, call: FakeCall, key: &str) -> Result<Option<R>, CallFailure> {
        self.script.lock().unwrap().log.push(call.clone());
        let hook = self.hook.lock().unwrap().take();
        if let Some(mut hook) = hook {
            hook(&call);
            let mut slot = self.hook.lock().unwrap();
            if slot.is_none() {
                *slot = Some(hook);
            }
        }
        let mut script = self.script.lock().unwrap();
        if let Some(at) = script
            .faults
            .iter()
            .position(|(only, _)| only.as_deref().is_none_or(|k| k == key))
        {
            let (_, fault) = script.faults.remove(at).unwrap();
            return Err(fault);
        }
        Ok(script.replies.get(key).cloned())
    }

    fn set_hook(&self, hook: Hook) {
        *self.hook.lock().unwrap() = Some(hook);
    }

    fn clear_hook(&self) {
        *self.hook.lock().unwrap() = None;
    }

    fn requests(&self) -> Vec<FakeCall> {
        self.script.lock().unwrap().log.clone()
    }

    fn clear_log(&self) {
        self.script.lock().unwrap().log.clear();
    }

    fn fault(&self, key: Option<String>, fault: CallFailure) {
        self.script.lock().unwrap().faults.push_back((key, fault));
    }

    fn reply(&self, key: String, reply: R) {
        self.script.lock().unwrap().replies.insert(key, reply);
    }
}

/// A transient failure after the request was sent (a timeout; no usage reported).
pub fn transient(text: &str) -> CallFailure {
    CallFailure {
        sent: true,
        status: None,
        kind: FailureKind::Timeout,
        quota: false,
        text: text.to_owned(),
        usage: None,
    }
}

/// A connect failure: no request byte left (`offline`).
pub fn offline() -> CallFailure {
    CallFailure {
        sent: false,
        status: None,
        kind: FailureKind::Connect,
        quota: false,
        text: "could not connect".to_owned(),
        usage: None,
    }
}

/// An HTTP error status from the provider.
pub fn status(code: u16, text: &str) -> CallFailure {
    CallFailure {
        sent: true,
        status: Some(code),
        kind: FailureKind::Status,
        quota: false,
        text: text.to_owned(),
        usage: None,
    }
}

/// A model refusal with its text.
pub fn refusal(text: &str) -> CallFailure {
    CallFailure {
        sent: true,
        status: None,
        kind: FailureKind::Refusal,
        quota: false,
        text: text.to_owned(),
        usage: None,
    }
}

/// A scripted model answer.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelScript {
    pub output: Json,
    pub usage: Option<Vec<(String, i64)>>,
    /// The response body the adapter would record; defaults to the output.
    pub response: Option<Json>,
}

fn model_key(stage: Stage, unit_key: &str) -> String {
    format!("{}:{unit_key}", stage.as_str())
}

/// The model provider stand-in (`fake-model`). Unscripted units answer `{}` with the default
/// usage.
pub struct FakeModel {
    core: Core<ModelScript>,
    default_usage: Mutex<Option<Vec<(String, i64)>>>,
}

impl Default for FakeModel {
    fn default() -> Self {
        Self {
            core: Core::new(),
            default_usage: Mutex::new(None),
        }
    }
}

impl FakeModel {
    pub const ID: &'static str = "fake-model";

    pub fn reply(&self, stage: Stage, unit_key: &str, output: Json) {
        self.script(
            stage,
            unit_key,
            ModelScript {
                output,
                usage: None,
                response: None,
            },
        );
    }

    pub fn script(&self, stage: Stage, unit_key: &str, script: ModelScript) {
        self.core.reply(model_key(stage, unit_key), script);
    }

    /// The usage reported by every reply that does not set its own.
    pub fn set_default_usage(&self, usage: Option<Vec<(String, i64)>>) {
        *self.default_usage.lock().unwrap() = usage;
    }

    /// A fault for the next call of any unit.
    pub fn fault(&self, fault: CallFailure) {
        self.core.fault(None, fault);
    }

    /// A fault for the next call of one unit.
    pub fn fault_for(&self, stage: Stage, unit_key: &str, fault: CallFailure) {
        self.core.fault(Some(model_key(stage, unit_key)), fault);
    }

    pub fn before_call(&self, hook: impl FnMut(&FakeCall) + Send + 'static) {
        self.core.set_hook(Box::new(hook));
    }

    pub fn clear_before_call(&self) {
        self.core.clear_hook();
    }

    pub fn requests(&self) -> Vec<FakeCall> {
        self.core.requests()
    }

    pub fn clear_requests(&self) {
        self.core.clear_log();
    }
}

impl ModelProvider for FakeModel {
    fn id(&self) -> String {
        Self::ID.to_owned()
    }

    fn call(&self, request: &ModelRequest) -> Result<ModelReply, CallFailure> {
        let call = FakeCall {
            port: "model",
            stage: Some(request.stage),
            unit_key: request.unit_key.clone(),
            images: request
                .images
                .iter()
                .map(|i| (i.sha256().to_owned(), i.long_edge()))
                .collect(),
            listing_images: request.listing_images.clone(),
            input: request.input.clone(),
            language: request.language.clone(),
            model: request.model.clone(),
            url: None,
        };
        let key = model_key(request.stage, &request.unit_key);
        let script = self.core.arrive(call, &key)?.unwrap_or(ModelScript {
            output: json!({}),
            usage: None,
            response: None,
        });
        let usage = script
            .usage
            .clone()
            .or_else(|| self.default_usage.lock().unwrap().clone());
        Ok(ModelReply {
            exchange: Exchange {
                request: json!({ "model": request.model, "input": request.input }),
                response: script.response.clone().unwrap_or(script.output.clone()),
            },
            output: script.output,
            usage,
        })
    }
}

/// The visual search stand-in (`fake-search`): one scripted result list for every image.
#[derive(Default)]
pub struct FakeVisualSearch {
    core: CoreHolder<Vec<SearchCandidate>>,
}

/// `Core` with a `Default` for the derive above.
struct CoreHolder<R>(Core<R>);

impl<R: Clone> Default for CoreHolder<R> {
    fn default() -> Self {
        Self(Core::new())
    }
}

impl FakeVisualSearch {
    pub const ID: &'static str = "fake-search";

    pub fn set_results(&self, candidates: Vec<SearchCandidate>) {
        self.core.0.reply(String::new(), candidates);
    }

    pub fn fault(&self, fault: CallFailure) {
        self.core.0.fault(None, fault);
    }

    pub fn before_call(&self, hook: impl FnMut(&FakeCall) + Send + 'static) {
        self.core.0.set_hook(Box::new(hook));
    }

    pub fn clear_before_call(&self) {
        self.core.0.clear_hook();
    }

    pub fn requests(&self) -> Vec<FakeCall> {
        self.core.0.requests()
    }

    pub fn clear_requests(&self) {
        self.core.0.clear_log();
    }
}

impl VisualSearchProvider for FakeVisualSearch {
    fn id(&self) -> String {
        Self::ID.to_owned()
    }

    fn search(&self, image: &ResearchImage) -> Result<SearchReply, CallFailure> {
        let call = FakeCall {
            port: "search",
            stage: Some(Stage::Find),
            unit_key: "cover".to_owned(),
            images: vec![(image.sha256().to_owned(), image.long_edge())],
            listing_images: Vec::new(),
            input: Json::Null,
            language: String::new(),
            model: String::new(),
            url: None,
        };
        let candidates = self.core.0.arrive(call, "")?.unwrap_or_default();
        Ok(SearchReply {
            exchange: Exchange {
                request: json!({ "image": { "image_sha256": image.sha256() } }),
                response: serde_json::to_value(&candidates).unwrap_or(Json::Null),
            },
            candidates,
            usage: None,
        })
    }
}

/// A scripted page.
#[derive(Debug, Clone, PartialEq)]
pub struct PageScript {
    pub obtained: bool,
    pub final_url: Option<String>,
    pub title: String,
    pub text: String,
    pub images: Vec<Vec<u8>>,
}

impl PageScript {
    pub fn page(title: &str, images: Vec<Vec<u8>>) -> Self {
        Self {
            obtained: true,
            final_url: None,
            title: title.to_owned(),
            text: format!("{title} — listing text"),
            images,
        }
    }

    pub fn unobtainable() -> Self {
        Self {
            obtained: false,
            final_url: None,
            title: String::new(),
            text: String::new(),
            images: Vec::new(),
        }
    }
}

/// The page reader stand-in (`fake-reader`), scripted per URL. An unscripted URL reads as a page
/// titled with its URL, without images.
#[derive(Default)]
pub struct FakePageReader {
    core: CoreHolder<PageScript>,
}

impl FakePageReader {
    pub const ID: &'static str = "fake-reader";

    pub fn set_page(&self, url: &str, page: PageScript) {
        self.core.0.reply(url.to_owned(), page);
    }

    pub fn fault(&self, fault: CallFailure) {
        self.core.0.fault(None, fault);
    }

    pub fn fault_for(&self, url: &str, fault: CallFailure) {
        self.core.0.fault(Some(url.to_owned()), fault);
    }

    pub fn before_call(&self, hook: impl FnMut(&FakeCall) + Send + 'static) {
        self.core.0.set_hook(Box::new(hook));
    }

    pub fn clear_before_call(&self) {
        self.core.0.clear_hook();
    }

    pub fn requests(&self) -> Vec<FakeCall> {
        self.core.0.requests()
    }

    pub fn clear_requests(&self) {
        self.core.0.clear_log();
    }
}

impl PageReader for FakePageReader {
    fn id(&self) -> String {
        Self::ID.to_owned()
    }

    fn read(&self, url: &str) -> Result<PageReply, CallFailure> {
        let call = FakeCall {
            port: "reader",
            stage: Some(Stage::Read),
            unit_key: url.to_owned(),
            images: Vec::new(),
            listing_images: Vec::new(),
            input: Json::Null,
            language: String::new(),
            model: String::new(),
            url: Some(url.to_owned()),
        };
        let page = self
            .core
            .0
            .arrive(call, url)?
            .unwrap_or_else(|| PageScript::page(url, Vec::new()));
        Ok(PageReply {
            exchange: Exchange {
                request: json!({ "url": url }),
                response: json!({
                    "obtained": page.obtained,
                    "final_url": page.final_url,
                    "title": page.title,
                    "text": page.text,
                }),
            },
            obtained: page.obtained,
            final_url: page.final_url,
            title: page.title,
            text: page.text,
            images: page.images,
            usage: None,
        })
    }
}

/// What [`FakeResearchHttp`] answers a matching request with.
pub type HttpScript = Result<crate::adapters::http::HttpReply, crate::adapters::http::HttpFailure>;

struct HttpRule {
    method: String,
    prefix: String,
    reply: HttpScript,
    once: bool,
}

#[derive(Default)]
struct HttpState {
    rules: Vec<HttpRule>,
    log: Vec<crate::adapters::http::HttpRequest>,
}

/// The HTTP seam stand-in (R8·U1): replays scripted replies (typically capture files, see
/// [`reply_from_capture`]) and records every request (method, URL, headers, every body part).
///
/// Matching: the first rule, in the order added, whose method equals the request's and whose URL
/// prefix the request's URL starts with. A `once` rule is used up by its match; an `always` rule
/// stays. An unscripted request gets a fixed 404 reply and is recorded; it never panics.
#[derive(Default)]
pub struct FakeResearchHttp {
    state: Mutex<HttpState>,
}

/// The body of the fixed reply to an unscripted request.
pub const UNSCRIPTED: &str = "not scripted";

impl FakeResearchHttp {
    fn add(&self, method: &str, prefix: &str, reply: HttpScript, once: bool) {
        self.state.lock().unwrap().rules.push(HttpRule {
            method: method.to_owned(),
            prefix: prefix.to_owned(),
            reply,
            once,
        });
    }

    /// Answers the next matching request once.
    pub fn once(&self, method: &str, prefix: &str, reply: HttpScript) {
        self.add(method, prefix, reply, true);
    }

    /// Answers every matching request.
    pub fn always(&self, method: &str, prefix: &str, reply: HttpScript) {
        self.add(method, prefix, reply, false);
    }

    /// Every request received, in order.
    pub fn requests(&self) -> Vec<crate::adapters::http::HttpRequest> {
        self.state.lock().unwrap().log.clone()
    }

    pub fn clear_requests(&self) {
        self.state.lock().unwrap().log.clear();
    }
}

impl crate::adapters::http::ResearchHttp for FakeResearchHttp {
    fn send(&self, request: &crate::adapters::http::HttpRequest) -> HttpScript {
        let mut state = self.state.lock().unwrap();
        state.log.push(request.clone());
        let found = state.rules.iter().position(|rule| {
            rule.method.eq_ignore_ascii_case(&request.method)
                && request.url.starts_with(&rule.prefix)
        });
        match found {
            Some(at) if state.rules[at].once => state.rules.remove(at).reply,
            Some(at) => state.rules[at].reply.clone(),
            None => Ok(crate::adapters::http::HttpReply {
                status: 404,
                headers: Vec::new(),
                body: UNSCRIPTED.as_bytes().to_vec(),
            }),
        }
    }
}

/// The reply a capture file recorded (`response.status`, `.headers`, `.body`): a string body is
/// its bytes, `null` (an omitted third-party body) is empty, any other JSON is serialized.
pub fn reply_from_capture(capture: &Json) -> crate::adapters::http::HttpReply {
    let response = &capture["response"];
    let headers = response["headers"]
        .as_object()
        .map(|map| {
            map.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_owned()))
                .collect()
        })
        .unwrap_or_default();
    let body = match &response["body"] {
        Json::Null => Vec::new(),
        Json::String(text) => text.as_bytes().to_vec(),
        other => serde_json::to_vec(other).unwrap_or_default(),
    };
    crate::adapters::http::HttpReply {
        status: u16::try_from(response["status"].as_u64().unwrap_or(0)).unwrap_or(0),
        headers,
        body,
    }
}
