//! `ModelProvider` over the OpenAI Responses API, in the request form `research-tool/src/openai.rs`
//! used for R1's captured calls. The instructions are the engine's, sent unchanged; the adapter
//! has none of its own. The key is read from the secrets port on every call.

use std::sync::Arc;

use base64::Engine;
use psm_core::ports::secrets::SecretStore;
use serde_json::{Value as Json, json};

use super::{errors, schema, usage::usage_of};
use crate::{
    adapters::http::{Body, HttpRequest, ResearchHttp, reply_record, request_record},
    config::DEFAULT_MODEL_PROVIDER,
    domain::{CallFailure, FailureKind},
    files::ResearchFiles,
    ports::{Exchange, ModelProvider, ModelReply, ModelRequest},
    service::MODEL_ACCOUNT,
};

pub const RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
/// R1's value (orchestrator decision, R8·U1).
pub const MAX_OUTPUT_TOKENS: u64 = 4_000;

pub struct OpenAiModel {
    http: Arc<dyn ResearchHttp>,
    secrets: Arc<dyn SecretStore>,
    files: ResearchFiles,
}

fn data_url(bytes: &[u8]) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

fn image_part(url: Json) -> Json {
    json!({ "type": "input_image", "image_url": url })
}

/// The concatenated `output_text` of a reply's message items.
fn output_text(body: &Json) -> Option<String> {
    let mut out = None::<String>;
    for item in body["output"].as_array().into_iter().flatten() {
        for content in item["content"].as_array().into_iter().flatten() {
            if content["type"] == "output_text"
                && let Some(text) = content["text"].as_str()
            {
                out.get_or_insert_with(String::new).push_str(text);
            }
        }
    }
    out
}

impl OpenAiModel {
    pub fn new(
        http: Arc<dyn ResearchHttp>,
        secrets: Arc<dyn SecretStore>,
        files: ResearchFiles,
    ) -> Self {
        Self {
            http,
            secrets,
            files,
        }
    }

    /// The body that is sent and the body as the `Exchange` records it (each image as
    /// `{"image_sha256": …}`).
    fn bodies(&self, request: &ModelRequest, schema: Json) -> Result<(Json, Json), CallFailure> {
        let mut input = request.input.clone();
        match input.as_object_mut() {
            Some(object) => {
                object.insert("language".into(), json!(request.language));
            }
            None => input = json!({ "input": input, "language": request.language }),
        }
        let text = json!({ "type": "input_text", "text": input.to_string() });
        let mut sent = vec![text.clone()];
        let mut recorded = vec![text];
        for image in &request.images {
            let bytes = image.bytes().map_err(|e| CallFailure {
                sent: true,
                status: None,
                kind: FailureKind::Status,
                quota: false,
                text: format!(
                    "the research copy of photo {} could not be read: {e}",
                    image.photo_id()
                ),
                usage: None,
            })?;
            sent.push(image_part(json!(data_url(&bytes))));
            recorded.push(image_part(json!({ "image_sha256": image.sha256() })));
        }
        for sha in &request.listing_images {
            // A listing image whose file is missing is left out; the rest is sent.
            if let Ok(bytes) = std::fs::read(self.files.path(sha)) {
                sent.push(image_part(json!(data_url(&bytes))));
                recorded.push(image_part(json!({ "image_sha256": sha })));
            }
        }
        let body = |content: Vec<Json>| {
            let mut body = json!({
                "input": [{ "role": "user", "content": content }],
                "instructions": request.instructions,
                "max_output_tokens": MAX_OUTPUT_TOKENS,
                "model": request.model,
                "store": false,
                "text": { "format": {
                    "type": "json_schema",
                    "name": request.stage.as_str(),
                    "strict": true,
                    "schema": schema,
                } },
            });
            if !request.effort.is_empty() {
                body["reasoning"] = json!({ "effort": request.effort });
            }
            body
        };
        Ok((body(sent), body(recorded)))
    }
}

impl ModelProvider for OpenAiModel {
    fn id(&self) -> String {
        DEFAULT_MODEL_PROVIDER.to_owned()
    }

    fn call(&self, request: &ModelRequest) -> Result<ModelReply, CallFailure> {
        let key = self
            .secrets
            .get(MODEL_ACCOUNT)
            .ok()
            .flatten()
            .unwrap_or_default();
        let schema = schema::schema_for(request);
        let (sent, recorded) = self.bodies(request, schema.clone())?;
        let http = HttpRequest::post(RESPONSES_URL, Body::Json(sent))
            .header("authorization", &format!("Bearer {}", key.trim()));
        let request_seen = request_record(&http, recorded);
        let reply = self
            .http
            .send(&http)
            .map_err(|failure| failure.into_call_failure())?;
        if !reply.ok() {
            return Err(errors::status_failure(&reply));
        }
        let body = reply.body_json();
        let usage = usage_of(&body);
        if let Some(text) = errors::refusal(&body) {
            return Err(errors::reply_failure(FailureKind::Refusal, text, usage));
        }
        if body["status"] != "completed" {
            let why = body["incomplete_details"]["reason"]
                .as_str()
                .unwrap_or("no reason given");
            return Err(errors::reply_failure(
                FailureKind::Schema,
                format!(
                    "the reply is {} ({why})",
                    body["status"].as_str().unwrap_or("not completed")
                ),
                usage,
            ));
        }
        let Some(text) = output_text(&body) else {
            return Err(errors::reply_failure(
                FailureKind::Schema,
                "the reply has no output text",
                usage,
            ));
        };
        let output: Json = match serde_json::from_str(&text) {
            Ok(output) => output,
            Err(e) => {
                return Err(errors::reply_failure(
                    FailureKind::Schema,
                    format!("the output is not JSON: {e}"),
                    usage,
                ));
            }
        };
        if let Err(why) = schema::check(&schema, &output) {
            return Err(errors::reply_failure(
                FailureKind::Schema,
                format!("the output does not fit the stage's schema: {why}"),
                usage,
            ));
        }
        Ok(ModelReply {
            output,
            usage,
            exchange: Exchange {
                request: request_seen,
                response: reply_record(&reply, body),
            },
        })
    }
}
