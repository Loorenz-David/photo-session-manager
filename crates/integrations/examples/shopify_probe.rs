//! P12 task 0: the Shopify grounding probe (SHOPIFY-VERIFY-0…7).
//!
//! Env: `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID`, `SHOPIFY_CLIENT_SECRET` (from `~/.psm_shopify.env`),
//! `SHOPIFY_API_VERSION` (default `2026-10`). Every printed line and every capture written to
//! `target/shopify_probe/` passes through `Redactor` first: the shop domain becomes
//! `example.myshopify.com`; the client id, the client secret and every access token become
//! placeholders; signed-URL query strings and staged-upload parameter values become placeholders.
//! Created ids go to the cleanup ledger (`target/shopify_probe/ledger.json`) before the next step.
//!
//! Usage: `cargo run -p psm-integrations --example shopify_probe -- <step>` where step is one of
//! `all`, `sv0`, `sv1`, `sv7`, `sv6`, `sv2`, `sv3`, `sv4`, `sv5`, `cleanup`, `gql <file>`.

use psm_integrations::commerce::shopify::transport::{HttpReply, HttpTransport, ReqwestTransport};
use serde_json::{Value as Json, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

const PROBE_SKU: &str = "PSM-PROBE-SKU-1";
const PROBE_BARCODE: &str = "4006381333931";
const PROBE_TITLE: &str = "PSM-PROBE product (Session Manager grounding probe)";

struct Redactor {
    shop: String,
    shop_handle: String,
    secrets: Mutex<Vec<(String, String)>>,
}

impl Redactor {
    fn add(&self, value: &str, placeholder: &str) {
        if value.len() >= 6 {
            self.secrets
                .lock()
                .unwrap()
                .push((value.to_owned(), placeholder.to_owned()));
        }
    }

    fn text(&self, input: &str) -> String {
        let mut out = input.to_owned();
        for (value, placeholder) in self.secrets.lock().unwrap().iter() {
            out = out.replace(value, placeholder);
        }
        out = out.replace(&self.shop, "example.myshopify.com");
        if self.shop_handle.len() >= 4 {
            out = out.replace(&self.shop_handle, "example");
        }
        out = redact_signed_queries(&out);
        redact_cdn_shop_path(&out)
    }

    fn json(&self, value: &Json) -> Json {
        let mut value = value.clone();
        redact_parameters(&mut value);
        serde_json::from_str(&self.text(&value.to_string())).unwrap_or(Json::Null)
    }
}

/// Every `?query` of an https URL to a storage host (signed URL) becomes `?<SIGNED_QUERY>`.
fn redact_signed_queries(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("https://") {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail
            .find(|c: char| c == '"' || c == '\\' || c.is_whitespace())
            .unwrap_or(tail.len());
        let url = &tail[..end];
        let signed = url.contains("X-Goog-")
            || url.contains("Signature")
            || url.contains("X-Amz-")
            || url.contains("GoogleAccessId");
        match url.split_once('?') {
            Some((base, _)) if signed => {
                out.push_str(base);
                out.push_str("?<SIGNED_QUERY>");
            }
            _ => out.push_str(url),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// `cdn.shopify.com/s/files/1/<a>/<b>/<c>/` → the shop path replaced by zeros.
fn redact_cdn_shop_path(input: &str) -> String {
    let marker = "/s/files/1/";
    let mut out = String::new();
    let mut rest = input;
    while let Some(at) = rest.find(marker) {
        out.push_str(&rest[..at + marker.len()]);
        let tail = &rest[at + marker.len()..];
        let mut parts = 0;
        let mut idx = 0;
        let bytes = tail.as_bytes();
        while parts < 3 {
            let seg_start = idx;
            while idx < bytes.len() && bytes[idx].is_ascii_digit() {
                idx += 1;
            }
            if idx == seg_start || idx >= bytes.len() || bytes[idx] != b'/' {
                idx = seg_start;
                break;
            }
            idx += 1;
            parts += 1;
        }
        if parts == 3 {
            out.push_str("0000/0000/0000/");
            rest = &tail[idx..];
        } else {
            rest = tail;
        }
    }
    out.push_str(rest);
    out
}

/// Staged-upload `parameters[].value` → `<PARAM_VALUE>` (shape kept), except `content_type`.
fn redact_parameters(value: &mut Json) {
    match value {
        Json::Object(map) => {
            if let Some(Json::Array(params)) = map.get_mut("parameters") {
                for p in params.iter_mut() {
                    if let Some(obj) = p.as_object_mut() {
                        let name = obj
                            .get("name")
                            .and_then(Json::as_str)
                            .unwrap_or("")
                            .to_owned();
                        if let Some(v) = obj.get_mut("value")
                            && !name.eq_ignore_ascii_case("content_type")
                        {
                            *v = Json::String(format!("<PARAM_VALUE:{name}>"));
                        }
                    }
                }
            }
            for v in map.values_mut() {
                redact_parameters(v);
            }
        }
        Json::Array(items) => items.iter_mut().for_each(redact_parameters),
        _ => {}
    }
}

struct Probe {
    shop: String,
    version: String,
    client_id: String,
    client_secret: String,
    http: ReqwestTransport,
    red: Redactor,
    token: Mutex<Option<String>>,
    dir: PathBuf,
}

impl Probe {
    fn new() -> Self {
        let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("missing env {k}"));
        let shop = env("SHOPIFY_SHOP");
        let shop = shop
            .trim()
            .trim_start_matches("https://")
            .trim_end_matches('/')
            .to_owned();
        let client_id = env("SHOPIFY_CLIENT_ID");
        let client_secret = env("SHOPIFY_CLIENT_SECRET");
        let version = std::env::var("SHOPIFY_API_VERSION").unwrap_or_else(|_| "2026-10".into());
        let red = Redactor {
            shop_handle: shop.trim_end_matches(".myshopify.com").to_owned(),
            shop: shop.clone(),
            secrets: Mutex::new(Vec::new()),
        };
        red.add(&client_secret, "<CLIENT_SECRET>");
        red.add(&client_id, "<CLIENT_ID>");
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/shopify_probe");
        fs::create_dir_all(dir.join("captures")).unwrap();
        Self {
            shop,
            version,
            client_id,
            client_secret,
            http: ReqwestTransport::new().expect("transport"),
            red,
            token: Mutex::new(None),
            dir,
        }
    }

    fn say(&self, line: &str) {
        println!("{}", self.red.text(line));
    }

    fn token_url(&self) -> String {
        format!("https://{}/admin/oauth/access_token", self.shop)
    }

    fn graphql_url(&self) -> String {
        format!(
            "https://{}/admin/api/{}/graphql.json",
            self.shop, self.version
        )
    }

    fn capture(&self, name: &str, request: Json, reply: &HttpReply) {
        let doc = json!({
            "request": self.red.json(&request),
            "response": { "status": reply.status, "body": self.red.json(&reply.body) },
        });
        let path = self.dir.join("captures").join(format!("{name}.json"));
        fs::write(&path, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
        self.say(&format!("  captured {name} (status {})", reply.status));
    }

    fn fetch_token(&self, capture_as: Option<&str>) -> String {
        let fields = [
            ("grant_type", "client_credentials"),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
        ];
        let reply = self
            .http
            .post_form(&self.token_url(), &fields)
            .expect("token request");
        if let Some(token) = reply.body.get("access_token").and_then(Json::as_str) {
            self.red.add(token, "<ACCESS_TOKEN>");
        }
        if let Some(name) = capture_as {
            self.capture(
                name,
                json!({
                    "method": "POST",
                    "url": self.token_url(),
                    "content_type": "application/x-www-form-urlencoded",
                    "fields": {"grant_type": "client_credentials", "client_id": self.client_id, "client_secret": self.client_secret},
                }),
                &reply,
            );
        }
        assert_eq!(
            reply.status,
            200,
            "token request failed: {}",
            self.red.json(&reply.body)
        );
        let token = reply.body["access_token"].as_str().unwrap().to_owned();
        *self.token.lock().unwrap() = Some(token.clone());
        token
    }

    fn token(&self) -> String {
        if let Some(t) = self.token.lock().unwrap().clone() {
            return t;
        }
        self.fetch_token(None)
    }

    /// One GraphQL call; captured under `name` when given. Returns the reply body.
    fn gql(&self, name: Option<&str>, query: &str, variables: Json) -> Json {
        let token = self.token();
        let mut attempt = 0;
        loop {
            let reply = self
                .http
                .post_graphql(&self.graphql_url(), &token, query, &variables)
                .expect("graphql");
            let throttled = reply.body["errors"]
                .as_array()
                .is_some_and(|e| e.iter().any(|e| e["extensions"]["code"] == "THROTTLED"));
            if (throttled || reply.status == 429) && attempt < 5 {
                attempt += 1;
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            if let Some(n) = name {
                self.capture(n, json!({"query": query, "variables": variables}), &reply);
            }
            if reply.status != 200 {
                self.say(&format!("  HTTP {}: {}", reply.status, reply.body));
            }
            if reply.body.get("errors").is_some() {
                self.say(&format!("  errors: {}", reply.body["errors"]));
            }
            return reply.body;
        }
    }

    fn ledger_path(&self) -> PathBuf {
        self.dir.join("ledger.json")
    }

    fn ledger(&self) -> Json {
        fs::read(self.ledger_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_else(|| json!({"products": [], "files": [], "definitions": [], "media": [], "deleted": []}))
    }

    fn ledger_add(&self, kind: &str, id: &str) {
        let mut l = self.ledger();
        let list = l[kind].as_array_mut().unwrap();
        if !list.iter().any(|v| v == id) {
            list.push(json!(id));
        }
        fs::write(
            self.ledger_path(),
            serde_json::to_string_pretty(&l).unwrap(),
        )
        .unwrap();
        self.say(&format!("  ledger + {kind} {id}"));
    }

    fn ledger_done(&self, kind: &str, id: &str) {
        let mut l = self.ledger();
        if let Some(list) = l[kind].as_array_mut() {
            list.retain(|v| v != id);
        }
        l["deleted"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind": kind, "id": id}));
        fs::write(
            self.ledger_path(),
            serde_json::to_string_pretty(&l).unwrap(),
        )
        .unwrap();
        self.say(&format!("  ledger - {kind} {id} (deleted)"));
    }

    fn write_note(&self, name: &str, value: Json) {
        let path = self.dir.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&self.red.json(&value)).unwrap(),
        )
        .unwrap();
    }

    fn read_note(&self, name: &str) -> Json {
        fs::read(self.dir.join(format!("{name}.json")))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or(Json::Null)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let probe = Probe::new();
    let step = args.first().map(String::as_str).unwrap_or("all");
    match step {
        "gql" => {
            let file = args.get(1).expect("gql <file>");
            let query = fs::read_to_string(file).unwrap();
            let vars = args
                .get(2)
                .map(|f| serde_json::from_str(&fs::read_to_string(f).unwrap()).unwrap())
                .unwrap_or(json!({}));
            let body = probe.gql(None, &query, vars);
            println!(
                "{}",
                probe
                    .red
                    .text(&serde_json::to_string_pretty(&body).unwrap())
            );
        }
        "token" => {
            let t = probe.fetch_token(None);
            probe.say(&format!("token ok ({} chars)", t.len()));
        }
        other => steps::run(&probe, other),
    }
    let leftovers = probe.ledger();
    let open: usize = ["products", "files", "definitions", "media"]
        .iter()
        .map(|k| leftovers[*k].as_array().map_or(0, |a| a.len()))
        .sum();
    if open > 0 {
        probe.say(&format!("LEDGER (not yet deleted): {}", leftovers));
    }
    let _ = Path::new(".");
}

mod steps {
    use super::*;

    use psm_integrations::commerce::shopify::graphql::DEFINITION_CREATE as DEF_CREATE;
    use psm_integrations::commerce::shopify::graphql::DEFINITION_DELETE as DEF_DELETE;
    use psm_integrations::commerce::shopify::graphql::DEFINITIONS as DEFS_PAGE;
    use psm_integrations::commerce::shopify::graphql::FILE_CREATE;
    use psm_integrations::commerce::shopify::graphql::FILE_DELETE;
    use psm_integrations::commerce::shopify::graphql::FILE_POLL;
    use psm_integrations::commerce::shopify::graphql::FILES_BY_NAME;
    use psm_integrations::commerce::shopify::graphql::ITEM_SET_IDENTITY as IDENTITY;
    use psm_integrations::commerce::shopify::graphql::JOB_POLL as JOB;
    use psm_integrations::commerce::shopify::graphql::MEDIA_POLL;
    use psm_integrations::commerce::shopify::graphql::METAFIELDS_READ;
    use psm_integrations::commerce::shopify::graphql::METAFIELDS_SET;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_CREATE_MEDIA as MEDIA_CREATE;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_DELETE;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_DELETE_MEDIA as DETACH;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_READ;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_REORDER_MEDIA as REORDER;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_SET;
    use psm_integrations::commerce::shopify::graphql::PRODUCT_UPDATE as TAGS;
    use psm_integrations::commerce::shopify::graphql::STAGED_UPLOADS_CREATE as STAGED;
    use psm_integrations::commerce::shopify::graphql::VARIANT_SEARCH;

    pub fn run(probe: &Probe, step: &str) {
        match step {
            "all" => {
                for s in [
                    "sv0",
                    "sv1",
                    "sv7",
                    "sv6",
                    "sv1values",
                    "sv5",
                    "sv2",
                    "sv3",
                    "sv4",
                    "tags",
                    "cleanup",
                ] {
                    run(probe, s);
                }
            }
            "sv0" => sv0(probe),
            "sv1" => sv1(probe),
            "sv7" => sv7(probe),
            "sv6" => sv6(probe),
            "identity" => identity(probe),
            "sv1values" => sv1_values(probe),
            "sv1reset" => sv1_values_with(probe, true),
            "sv5" => sv5(probe),
            "sv2" => sv2(probe),
            "sv3" => sv3(probe),
            "sv4" => sv4(probe),
            "tags" => tags(probe),
            "cleanup" => cleanup(probe),
            "adapter_smoke" => adapter_smoke(probe),
            other => panic!("unknown step {other}"),
        }
    }

    fn state(probe: &Probe) -> Json {
        let s = probe.read_note("state");
        if s.is_null() { json!({}) } else { s }
    }

    fn set_state(probe: &Probe, key: &str, value: Json) {
        let mut s = state(probe);
        s[key] = value;
        probe.write_note("state", s);
    }

    fn product_id(probe: &Probe) -> String {
        state(probe)["product_id"]
            .as_str()
            .expect("run sv7 first")
            .to_owned()
    }

    fn sv0(probe: &Probe) {
        probe.say("SV-0 token request (form body)");
        probe.fetch_token(Some("sv0_token"));
        probe.say("SV-0 wrong secret");
        let fields = [
            ("grant_type", "client_credentials"),
            ("client_id", probe.client_id.as_str()),
            ("client_secret", "psm-probe-deliberately-wrong-secret"),
        ];
        let reply = probe
            .http
            .post_form(&probe.token_url(), &fields)
            .expect("token");
        probe.capture(
            "sv0_token_wrong_secret",
            json!({"method": "POST", "url": probe.token_url(), "content_type": "application/x-www-form-urlencoded",
                   "fields": {"grant_type": "client_credentials", "client_id": probe.client_id, "client_secret": "<WRONG_SECRET>"}}),
            &reply,
        );
        probe.say("SV-0 GraphQL with a wrong token");
        let q = "query Shop { shop { name } }";
        let reply = probe
            .http
            .post_graphql(
                &probe.graphql_url(),
                "shpat_psm_probe_wrong_token_0000000000",
                q,
                &json!({}),
            )
            .expect("graphql");
        probe.capture(
            "sv0_graphql_wrong_token",
            json!({"query": q, "variables": {}, "access_token": "<WRONG_TOKEN>"}),
            &reply,
        );
        probe.say("SV-0 token request (JSON body)");
        let reply = probe
            .http
            .post_graphql(&probe.token_url(), "", "", &json!({}))
            .ok();
        if let Some(r) = reply {
            probe.say(&format!(
                "  json-shaped POST to the token endpoint: status {}",
                r.status
            ));
        }
    }

    fn all_definitions(probe: &Probe, capture_prefix: &str) -> Vec<Json> {
        let mut out = Vec::new();
        let mut after = Json::Null;
        let mut page = 1;
        loop {
            let body = probe.gql(
                Some(&format!("{capture_prefix}_p{page}")),
                DEFS_PAGE,
                json!({"after": after}),
            );
            let conn = &body["data"]["metafieldDefinitions"];
            out.extend(conn["nodes"].as_array().cloned().unwrap_or_default());
            if conn["pageInfo"]["hasNextPage"] != json!(true) {
                break;
            }
            after = conn["pageInfo"]["endCursor"].clone();
            page += 1;
        }
        out
    }

    fn sv1(probe: &Probe) {
        probe.say("SV-1 definitions before scratch");
        let defs = all_definitions(probe, "sv1_definitions_before");
        probe.say(&format!("  {} definitions", defs.len()));
        let has = |t: &str| defs.iter().any(|d| d["type"]["name"] == t);
        let mut scratch: Vec<(String, String, String, Json)> = Vec::new();
        for t in [
            "number_decimal",
            "weight",
            "volume",
            "boolean",
            "date",
            "date_time",
            "color",
            "money",
            "rating",
            "url",
            "rich_text_field",
            "file_reference",
            "product_reference",
            "list.product_reference",
            "multi_line_text_field",
            "number_integer",
            "dimension",
            "metaobject_reference",
            "list.metaobject_reference",
        ] {
            if !has(t) {
                let key = format!("psm_probe_{}", t.replace('.', "_"));
                let validations = match t {
                    "file_reference" => {
                        json!([{"name": "file_type_options", "value": "[\"Image\",\"Video\"]"}])
                    }
                    "rating" => {
                        json!([{"name": "scale_min", "value": "1.0"}, {"name": "scale_max", "value": "5.0"}])
                    }
                    _ => json!([]),
                };
                scratch.push(("custom".into(), key, t.into(), validations));
            }
        }
        // A second images-only file list (C4(v)) and a severity-choice condition field (C4 cracks clause).
        scratch.push((
            "custom".into(),
            "psm_probe_list_file_reference".into(),
            "list.file_reference".into(),
            json!([{"name": "file_type_options", "value": "[\"Image\"]"}]),
        ));
        scratch.push(("custom".into(), "condition_psm_probe_cracks".into(), "single_line_text_field".into(),
            json!([{"name": "choices", "value": "[\"N/A\",\"None\",\"Minor\",\"Moderate\",\"Major\"]"}])));
        for (ns, key, t, validations) in scratch {
            let name = if key.starts_with("psm_probe") {
                key.clone()
            } else {
                format!("psm_probe_{key}")
            };
            let body = probe.gql(Some(&format!("sv1_scratch_create_{key}")), DEF_CREATE, json!({"definition": {
                "namespace": ns, "key": key, "name": name, "ownerType": "PRODUCT", "type": t, "validations": validations,
                "description": "Session Manager grounding probe (temporary; deleted by the probe)"
            }}));
            if let Some(id) =
                body["data"]["metafieldDefinitionCreate"]["createdDefinition"]["id"].as_str()
            {
                probe.ledger_add("definitions", id);
            } else {
                probe.say(&format!(
                    "  create {ns}.{key} ({t}) failed: {}",
                    body["data"]["metafieldDefinitionCreate"]["userErrors"]
                ));
            }
        }
        probe.say("SV-1 definitions after scratch (C4 fixture)");
        let after = all_definitions(probe, "sv1_definitions");
        probe.say(&format!("  {} definitions", after.len()));
        let mut units = json!({});
        for t in [
            "dimension",
            "weight",
            "volume",
            "rating",
            "money",
            "rich_text_field",
        ] {
            let found = probe.gql(None,
                "query T($n: String!) { metafieldDefinitionTypes { name category supportedValidations { name type } } }",
                json!({"n": t}));
            let entry = found["data"]["metafieldDefinitionTypes"]
                .as_array()
                .and_then(|a| a.iter().find(|x| x["name"] == t).cloned());
            units[t] = entry.unwrap_or(Json::Null);
        }
        probe.write_note("sv1_type_validations", units);
    }

    fn sv7(probe: &Probe) {
        probe.say("SV-7 create (productSet synchronous, DRAFT, default variant SKU + barcode)");
        let input = json!({
            "title": PROBE_TITLE,
            "status": "DRAFT",
            "productOptions": [{"name": "Title", "values": [{"name": "Default Title"}]}],
            "variants": [{"optionValues": [{"optionName": "Title", "name": "Default Title"}], "sku": PROBE_SKU, "barcodes": [{"value": PROBE_BARCODE}]}],
        });
        let body = probe.gql(Some("sv7_create"), PRODUCT_SET, json!({"input": input}));
        let id = body["data"]["productSet"]["product"]["id"]
            .as_str()
            .map(str::to_owned);
        match id {
            Some(id) => {
                probe.ledger_add("products", &id);
                set_state(probe, "product_id", json!(id));
                set_state(probe, "created_at_ms", json!(now_ms()));
                probe.gql(Some("sv7_create_readback"), PRODUCT_READ, json!({"id": id}));
            }
            None => panic!("create failed: {}", probe.red.json(&body)),
        }
    }

    fn now_ms() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    fn search(probe: &Probe, name: &str, q: &str) -> usize {
        let body = probe.gql(Some(name), VARIANT_SEARCH, json!({"query": q}));
        let n = body["data"]["productVariants"]["nodes"]
            .as_array()
            .map_or(0, |a| a.len());
        probe.say(&format!("  search {q:?} -> {n} variant(s)"));
        n
    }

    fn sv6(probe: &Probe) {
        probe.say("SV-6 search by SKU after create: now, +5 s, +30 s");
        let started = Instant::now();
        let created = state(probe)["created_at_ms"].as_u64().unwrap_or(0) as u128;
        let q = format!("sku:\"{PROBE_SKU}\"");
        let mut log = Vec::new();
        let n0 = search(probe, "sv6_search_immediate", &q);
        log.push(json!({"label": "immediate", "ms_after_create": now_ms() - created, "found": n0}));
        while started.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(200));
        }
        let n5 = search(probe, "sv6_search_5s", &q);
        log.push(json!({"label": "+5 s", "ms_after_create": now_ms() - created, "found": n5}));
        while started.elapsed() < Duration::from_secs(30) {
            std::thread::sleep(Duration::from_millis(500));
        }
        let n30 = search(probe, "sv6_search_30s", &q);
        log.push(json!({"label": "+30 s", "ms_after_create": now_ms() - created, "found": n30}));
        let lower = format!("sku:\"{}\"", PROBE_SKU.to_lowercase());
        let nl = search(probe, "sv6_search_other_case", &lower);
        log.push(json!({"label": "other case", "found": nl}));
        let nu = search(probe, "sv6_search_unquoted", &format!("sku:{PROBE_SKU}"));
        log.push(json!({"label": "unquoted", "found": nu}));
        let nz = search(probe, "sv6_search_none", "sku:\"PSM-PROBE-NO-SUCH-SKU\"");
        log.push(json!({"label": "no match", "found": nz}));
        let np = search(probe, "sv6_search_prefix", "sku:\"PSM-PROBE-SKU\"");
        log.push(json!({"label": "prefix only", "found": np}));
        probe.write_note("sv6_log", json!(log));
    }

    fn identity(probe: &Probe) {
        let id = product_id(probe);
        let body = probe.gql(None, PRODUCT_READ, json!({"id": id}));
        let variant = body["data"]["product"]["variants"]["nodes"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        probe.say("SV-7 identity: SKU only");
        probe.gql(Some("sv7_identity_sku_only"), IDENTITY, json!({"productId": id, "variants": [{"id": variant, "inventoryItem": {"sku": "PSM-PROBE-SKU-2"}}]}));
        probe.gql(
            Some("sv7_identity_sku_only_readback"),
            PRODUCT_READ,
            json!({"id": id}),
        );
        probe.say("SV-7 identity: barcode only");
        probe.gql(Some("sv7_identity_barcode_only"), IDENTITY, json!({"productId": id, "variants": [{"id": variant, "barcodes": [{"value": "4006381333948"}]}]}));
        probe.gql(
            Some("sv7_identity_barcode_only_readback"),
            PRODUCT_READ,
            json!({"id": id}),
        );
        probe.say("SV-7 identity: both back to the originals");
        probe.gql(Some("sv7_identity_both"), IDENTITY, json!({"productId": id, "variants": [{"id": variant, "inventoryItem": {"sku": PROBE_SKU}, "barcodes": [{"value": PROBE_BARCODE}]}]}));
        probe.gql(
            Some("sv7_identity_both_readback"),
            PRODUCT_READ,
            json!({"id": id}),
        );
        set_state(probe, "variant_id", json!(variant));
    }

    fn values() -> Vec<(&'static str, &'static str, &'static str, String)> {
        let rich = json!({"type": "root", "children": [
            {"type": "paragraph", "children": [{"type": "text", "value": "First paragraph & more"}]},
            {"type": "paragraph", "children": [{"type": "text", "value": "Second <b> \"q\""}]}]}).to_string();
        vec![
            (
                "custom",
                "extention_type",
                "single_line_text_field",
                "Two Outside".into(),
            ),
            (
                "beyo",
                "fx_text",
                "single_line_text_field",
                "Hej världen ✓".into(),
            ),
            (
                "custom",
                "testing_listing",
                "list.single_line_text_field",
                "[\"Completely restored\",\"Light wear consistent with age and use\"]".into(),
            ),
            (
                "beyo",
                "fx_list_text",
                "list.single_line_text_field",
                "[\"teak\",\"oak\"]".into(),
            ),
            (
                "beyo",
                "fx_text_long",
                "multi_line_text_field",
                "line one\nline two\n\nsecond paragraph".into(),
            ),
            (
                "custom",
                "psm_probe_rich_text_field",
                "rich_text_field",
                rich,
            ),
            ("beyo", "fx_integer", "number_integer", "42".into()),
            ("beyo", "fx_decimal", "number_decimal", "12.50".into()),
            (
                "beyo",
                "fx_dimension",
                "dimension",
                "{\"value\":45.5,\"unit\":\"cm\"}".into(),
            ),
            (
                "beyo",
                "fx_weight",
                "weight",
                "{\"value\":2.5,\"unit\":\"kg\"}".into(),
            ),
            (
                "beyo",
                "fx_volume",
                "volume",
                "{\"value\":1.5,\"unit\":\"l\"}".into(),
            ),
            ("beyo", "fx_boolean", "boolean", "true".into()),
            ("beyo", "fx_date", "date", "2026-10-26".into()),
            (
                "beyo",
                "fx_datetime",
                "date_time",
                "2026-10-26T10:00:00Z".into(),
            ),
            ("beyo", "fx_color", "color", "#a1b2c3".into()),
            (
                "beyo",
                "fx_rating",
                "rating",
                "{\"value\":\"4.5\",\"scale_min\":\"1.0\",\"scale_max\":\"5.0\"}".into(),
            ),
            (
                "beyo",
                "fx_money",
                "money",
                "{\"amount\":\"12.50\",\"currency_code\":\"USD\"}".into(),
            ),
            (
                "beyo",
                "fx_url",
                "url",
                "https://example.com/item?x=1".into(),
            ),
        ]
    }

    fn read_metafields(probe: &Probe, name: &str) -> Vec<Json> {
        let id = product_id(probe);
        let body = probe.gql(
            Some(name),
            METAFIELDS_READ,
            json!({"id": id, "after": null}),
        );
        body["data"]["product"]["metafields"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    }

    fn sv1_values(probe: &Probe) {
        sv1_values_with(probe, false)
    }

    fn sv1_values_with(probe: &Probe, only_values: bool) {
        let id = product_id(probe);
        if only_values {
            let entries: Vec<Json> = values().into_iter().map(|(ns, key, t, v)| json!({"ownerId": id, "namespace": ns, "key": key, "type": t, "value": v})).collect();
            probe.gql(
                Some("sv1_values_set"),
                METAFIELDS_SET,
                json!({"metafields": entries}),
            );
            read_metafields(probe, "sv1_values_readback");
            return;
        }
        probe.say("SV-1 unit codes: one write per candidate unit");
        let mut units = json!({});
        for (key, t, candidates) in [
            (
                "fx_dimension",
                "dimension",
                vec![
                    "mm",
                    "cm",
                    "m",
                    "in",
                    "ft",
                    "yd",
                    "MILLIMETERS",
                    "CENTIMETERS",
                    "METERS",
                    "INCHES",
                    "FEET",
                    "YARDS",
                ],
            ),
            (
                "fx_weight",
                "weight",
                vec![
                    "oz",
                    "lb",
                    "g",
                    "kg",
                    "OUNCES",
                    "POUNDS",
                    "GRAMS",
                    "KILOGRAMS",
                ],
            ),
            (
                "fx_volume",
                "volume",
                vec![
                    "ml",
                    "cl",
                    "l",
                    "m3",
                    "us_fl_oz",
                    "us_pt",
                    "us_qt",
                    "us_gal",
                    "imp_fl_oz",
                    "imp_pt",
                    "imp_qt",
                    "imp_gal",
                    "MILLILITERS",
                    "LITERS",
                ],
            ),
        ] {
            let mut accepted = Vec::new();
            for unit in candidates {
                let v = format!("{{\"value\":1.5,\"unit\":\"{unit}\"}}");
                let body = probe.gql(None, METAFIELDS_SET, json!({"metafields": [{"ownerId": id, "namespace": "beyo", "key": key, "type": t, "value": v}]}));
                let errs = &body["data"]["metafieldsSet"]["userErrors"];
                let stored = body["data"]["metafieldsSet"]["metafields"][0]["value"].clone();
                accepted.push(json!({"sent": unit, "accepted": errs.as_array().is_some_and(|e| e.is_empty()), "stored": stored, "errors": errs}));
            }
            units[t] = json!(accepted);
        }
        probe.write_note("sv1_units", units);
        probe.say("SV-1 one value of every type");
        let entries: Vec<Json> = values().into_iter().map(|(ns, key, t, v)| json!({"ownerId": id, "namespace": ns, "key": key, "type": t, "value": v})).collect();
        probe.gql(
            Some("sv1_values_set"),
            METAFIELDS_SET,
            json!({"metafields": entries}),
        );
        read_metafields(probe, "sv1_values_readback");
        if only_values {
            return;
        }
        probe.say("SV-1 foreign namespaces");
        for (i, (ns, key, t, v)) in [
            (
                "reviews",
                "rating",
                "rating",
                "{\"value\":\"4.0\",\"scale_min\":\"1.0\",\"scale_max\":\"5.0\"}",
            ),
            (
                "shopify--discovery--product_recommendation",
                "related_products_display",
                "single_line_text_field",
                "ahead",
            ),
            (
                "globo--filter--product_recommendation",
                "psm_probe_text",
                "single_line_text_field",
                "x",
            ),
            ("app", "item_location", "single_line_text_field", "H1"),
        ]
        .into_iter()
        .enumerate()
        {
            probe.gql(Some(&format!("sv1_foreign_{i}_{}", ns.split("--").next().unwrap())), METAFIELDS_SET,
                json!({"metafields": [{"ownerId": id, "namespace": ns, "key": key, "type": t, "value": v}]}));
        }
    }

    fn sv5(probe: &Probe) {
        let id = product_id(probe);
        probe.say("SV-5 limit: 26 then 25 entries");
        let entries = |n: usize| -> Vec<Json> {
            (0..n).map(|i| json!({"ownerId": id, "namespace": "psm_probe", "key": format!("limit_{i:02}"), "type": "single_line_text_field", "value": format!("v{i}")})).collect()
        };
        probe.gql(
            Some("sv5_limit_26"),
            METAFIELDS_SET,
            json!({"metafields": entries(26)}),
        );
        probe.gql(
            Some("sv5_limit_25"),
            METAFIELDS_SET,
            json!({"metafields": entries(25)}),
        );
        probe.say("SV-5 atomicity: one invalid entry of three");
        let body = probe.gql(Some("sv5_partial_one_invalid"), METAFIELDS_SET, json!({"metafields": [
            {"ownerId": id, "namespace": "beyo", "key": "fx_text", "type": "single_line_text_field", "value": "sv5 changed"},
            {"ownerId": id, "namespace": "beyo", "key": "fx_integer", "type": "number_integer", "value": "not-a-number"},
            {"ownerId": id, "namespace": "beyo", "key": "fx_boolean", "type": "boolean", "value": "false"},
        ]}));
        let _ = body;
        read_metafields(probe, "sv5_partial_one_invalid_readback");
        probe.say("SV-5 two invalid entries of three");
        probe.gql(Some("sv5_partial_two_invalid"), METAFIELDS_SET, json!({"metafields": [
            {"ownerId": id, "namespace": "beyo", "key": "fx_integer", "type": "number_integer", "value": "4.5"},
            {"ownerId": id, "namespace": "beyo", "key": "fx_text", "type": "single_line_text_field", "value": "sv5 second"},
            {"ownerId": id, "namespace": "beyo", "key": "fx_color", "type": "color", "value": "not-a-colour"},
        ]}));
        read_metafields(probe, "sv5_partial_two_invalid_readback");
    }

    fn staged(
        probe: &Probe,
        name: &str,
        filename: &str,
        resource: &str,
        size: usize,
    ) -> (String, String, Vec<(String, String)>) {
        let body = probe.gql(Some(name), STAGED, json!({"input": [{"resource": resource, "filename": filename, "mimeType": "image/jpeg", "httpMethod": "PUT", "fileSize": size.to_string()}]}));
        let t = &body["data"]["stagedUploadsCreate"]["stagedTargets"][0];
        let params = t["parameters"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|p| {
                (
                    p["name"].as_str().unwrap().to_owned(),
                    p["value"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        (
            t["url"].as_str().expect("staged url").to_owned(),
            t["resourceUrl"].as_str().expect("resource url").to_owned(),
            params,
        )
    }

    fn put(probe: &Probe, name: &str, url: &str, params: &[(String, String)], bytes: &[u8]) {
        let reply = probe.http.put_bytes(url, params, bytes).expect("put");
        let headers: serde_json::Map<String, Json> =
            params.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        probe.capture(
            name,
            json!({"method": "PUT", "url": url, "headers": headers, "body_bytes": bytes.len()}),
            &reply,
        );
        assert!(reply.status < 300, "upload failed {}", reply.status);
    }

    fn poll_media(probe: &Probe, name: &str, id: &str) -> Json {
        for i in 0..40 {
            let first = format!("{name}_first");
            let body = probe.gql(
                if i == 0 { Some(first.as_str()) } else { None },
                MEDIA_POLL,
                json!({"id": id}),
            );
            let status = body["data"]["node"]["status"]
                .as_str()
                .unwrap_or("")
                .to_owned();
            if status == "READY" || status == "FAILED" {
                probe.gql(
                    Some(&format!("{name}_ready")),
                    MEDIA_POLL,
                    json!({"id": id}),
                );
                probe.say(&format!("  media {id} {status} after {i} polls"));
                return body;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        panic!("media {id} never READY");
    }

    fn sv2(probe: &Probe) {
        let id = product_id(probe);
        let dir = probe.dir.clone();
        let jpegs: Vec<Vec<u8>> = (1..=3)
            .map(|i| fs::read(dir.join(format!("probe{i}.jpg"))).unwrap())
            .collect();
        let names = [
            "psm-probe-ab12cd34ef56.jpg",
            "psm-probe-ab12cd34ef56.jpg",
            "psm-probe-0123456789ab.jpg",
        ];
        let mut media: Vec<String> = Vec::new();
        let before = probe.gql(None, PRODUCT_READ, json!({"id": id}));
        for n in before["data"]["product"]["media"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            let m = n["id"].as_str().unwrap().to_owned();
            probe.ledger_add("media", &m);
            probe.say(&format!(
                "  removing stray media {m} from an earlier attempt"
            ));
            let b = probe.gql(None, FILE_DELETE, json!({"ids": [m]}));
            if b["data"]["fileDelete"]["deletedFileIds"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
            {
                probe.ledger_done("media", &m);
            }
        }
        for (i, (name, bytes)) in names.iter().zip(&jpegs).enumerate() {
            probe.say(&format!("SV-2 upload {} ({name})", i + 1));
            let (url, resource, params) = staged(
                probe,
                &format!("sv2_staged_{}", i + 1),
                name,
                "IMAGE",
                bytes.len(),
            );
            put(probe, &format!("sv2_put_{}", i + 1), &url, &params, bytes);
            let body = probe.gql(Some(&format!("sv2_media_create_{}", i + 1)), MEDIA_CREATE, json!({
                "product": {"id": id}, "media": [{"originalSource": resource, "alt": format!("psm probe {}", i + 1), "mediaContentType": "IMAGE"}]}));
            let mid = body["data"]["productUpdate"]["product"]["newest"]["nodes"][0]["id"]
                .as_str()
                .expect("media id")
                .to_owned();
            let all: Vec<String> = body["data"]["productUpdate"]["product"]["all"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n["id"].as_str().unwrap().to_owned())
                .collect();
            probe.say(&format!(
                "  newest = {mid}; full list last = {:?}; previously known = {:?}",
                all.last(),
                media
            ));
            for m in &all {
                probe.ledger_add("media", m);
            }
            probe.ledger_add("media", &mid);
            poll_media(probe, &format!("sv2_media_poll_{}", i + 1), &mid);
            media.push(mid);
        }
        set_state(probe, "media", json!(media));
        probe.gql(Some("sv2_item_get"), PRODUCT_READ, json!({"id": id}));
        probe.gql(
            Some("sv2_files_by_name"),
            FILES_BY_NAME,
            json!({"query": "filename:psm-probe-ab12cd34ef56"}),
        );
    }

    fn poll_file(probe: &Probe, name: &str, id: &str) {
        for i in 0..40 {
            let body = probe.gql(Some(&format!("{name}_{i}")), FILE_POLL, json!({"id": id}));
            let status = body["data"]["node"]["fileStatus"]
                .as_str()
                .unwrap_or("")
                .to_owned();
            if status == "READY" || status == "FAILED" {
                probe.say(&format!("  file {id} {status} after {} polls", i + 1));
                return;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        panic!("file {id} never READY");
    }

    fn sv3(probe: &Probe) {
        let id = product_id(probe);
        let bytes = fs::read(probe.dir.join("probe1.jpg")).unwrap();
        let mut files = Vec::new();
        for i in 1..=2 {
            probe.say(&format!("SV-3 fileCreate {i} (same filename, APPEND_UUID)"));
            let fname = "psm-probe-cafe00000001.jpg";
            let (url, resource, params) = staged(
                probe,
                &format!("sv3_staged_{i}"),
                fname,
                "FILE",
                bytes.len(),
            );
            put(probe, &format!("sv3_put_{i}"), &url, &params, &bytes);
            let body = probe.gql(Some(&format!("sv3_file_create_{i}")), FILE_CREATE, json!({"files": [{
                "originalSource": resource, "filename": fname, "contentType": "IMAGE", "alt": "psm probe condition", "duplicateResolutionMode": "APPEND_UUID"}]}));
            let fid = body["data"]["fileCreate"]["files"][0]["id"]
                .as_str()
                .expect("file id")
                .to_owned();
            probe.ledger_add("files", &fid);
            poll_file(probe, &format!("sv3_file_poll_{i}"), &fid);
            files.push(fid);
        }
        set_state(probe, "files", json!(files));
        probe.gql(
            Some("sv3_files_by_name"),
            FILES_BY_NAME,
            json!({"query": "filename:psm-probe-cafe00000001"}),
        );
        probe.say("SV-3 media_list write (damage report) with the two file ids");
        probe.gql(Some("sv3_damage_report_set"), METAFIELDS_SET, json!({"metafields": [{"ownerId": id, "namespace": "custom", "key": "damage_report", "type": "list.file_reference", "value": json!(files).to_string()}]}));
        read_metafields(probe, "sv3_damage_report_readback");
    }

    fn sv4(probe: &Probe) {
        let id = product_id(probe);
        let media: Vec<String> = state(probe)["media"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m.as_str().unwrap().to_owned())
            .collect();
        probe.say("SV-4 reorder: reverse the three media, newPosition = index");
        let reversed: Vec<&String> = media.iter().rev().collect();
        let moves: Vec<Json> = reversed
            .iter()
            .enumerate()
            .map(|(i, m)| json!({"id": m, "newPosition": i.to_string()}))
            .collect();
        let body = probe.gql(
            Some("sv4_reorder"),
            REORDER,
            json!({"id": id, "moves": moves}),
        );
        if let Some(job) = body["data"]["productReorderMedia"]["job"]["id"].as_str() {
            for i in 0..30 {
                let b = probe.gql(Some(&format!("sv4_job_poll_{i}")), JOB, json!({"id": job}));
                if b["data"]["job"]["done"] == json!(true) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        probe.gql(
            Some("sv4_reorder_readback"),
            PRODUCT_READ,
            json!({"id": id}),
        );
        probe.say("SV-4 reorder with a 1-based position (base check)");
        let moves1: Vec<Json> = media
            .iter()
            .enumerate()
            .map(|(i, m)| json!({"id": m, "newPosition": (i + 1).to_string()}))
            .collect();
        let body = probe.gql(
            Some("sv4_reorder_one_based"),
            REORDER,
            json!({"id": id, "moves": moves1}),
        );
        if let Some(job) = body["data"]["productReorderMedia"]["job"]["id"].as_str() {
            for _ in 0..30 {
                let b = probe.gql(None, JOB, json!({"id": job}));
                if b["data"]["job"]["done"] == json!(true) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        probe.gql(
            Some("sv4_reorder_one_based_readback"),
            PRODUCT_READ,
            json!({"id": id}),
        );
        probe.say("SV-4 remove one media from the product (fileUpdate referencesToRemove), twice");
        let victim = media[0].clone();
        probe.gql(
            Some("sv4_delete_media"),
            DETACH,
            json!({"files": [{"id": victim, "referencesToRemove": [id]}]}),
        );
        probe.gql(
            Some("sv4_delete_media_again"),
            DETACH,
            json!({"files": [{"id": victim, "referencesToRemove": [id]}]}),
        );
        probe.gql(Some("sv4_delete_readback"), PRODUCT_READ, json!({"id": id}));
        probe.say("SV-4 fileDelete of the removed media, twice");
        probe.gql(
            Some("sv4_file_delete"),
            FILE_DELETE,
            json!({"ids": [victim]}),
        );
        probe.ledger_done("media", &victim);
        probe.gql(
            Some("sv4_file_delete_again"),
            FILE_DELETE,
            json!({"ids": [victim]}),
        );
        probe.gql(
            Some("sv4_delete_media_unknown"),
            DETACH,
            json!({"files": [{"id": victim, "referencesToRemove": [id]}]}),
        );
    }

    fn tags(probe: &Probe) {
        let id = product_id(probe);
        probe.say("tags / title / description write");
        probe.gql(
            Some("tags_update"),
            TAGS,
            json!({"product": {"id": id, "tags": ["a", "b"]}}),
        );
        probe.gql(Some("description_update"), TAGS, json!({"product": {"id": id, "descriptionHtml": "<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>"}}));
    }

    fn cleanup(probe: &Probe) {
        probe.say("cleanup");
        let l = probe.ledger();
        for id in l["products"].as_array().cloned().unwrap_or_default() {
            let id = id.as_str().unwrap();
            let body = probe.gql(
                Some("cleanup_product_delete"),
                PRODUCT_DELETE,
                json!({"id": id}),
            );
            if body["data"]["productDelete"]["deletedProductId"].is_string() {
                probe.ledger_done("products", id);
            }
        }
        for kind in ["media", "files"] {
            for id in l[kind].as_array().cloned().unwrap_or_default() {
                let id = id.as_str().unwrap();
                let body = probe.gql(
                    Some(&format!("cleanup_{kind}_delete")),
                    FILE_DELETE,
                    json!({"ids": [id]}),
                );
                let deleted = body["data"]["fileDelete"]["deletedFileIds"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty());
                let gone = body["data"]["fileDelete"]["userErrors"]
                    .as_array()
                    .is_some_and(|e| e.iter().any(|e| e["code"] == "FILE_DOES_NOT_EXIST"));
                if deleted || gone {
                    probe.ledger_done(kind, id);
                }
            }
        }
        for id in l["definitions"].as_array().cloned().unwrap_or_default() {
            let id = id.as_str().unwrap();
            let body = probe.gql(None, DEF_DELETE, json!({"id": id}));
            if body["data"]["metafieldDefinitionDelete"]["deletedDefinitionId"].is_string() {
                probe.ledger_done("definitions", id);
            }
        }
        let left = probe.gql(
            Some("cleanup_files_left"),
            FILES_BY_NAME,
            json!({"query": "filename:psm-probe*"}),
        );
        probe.say(&format!(
            "  files still matching psm-probe: {}",
            left["data"]["files"]["nodes"]
                .as_array()
                .map_or(0, |a| a.len())
        ));
        let left = probe.gql(
            Some("cleanup_products_left"),
            VARIANT_SEARCH,
            json!({"query": "sku:PSM-PROBE*"}),
        );
        probe.say(&format!(
            "  variants still matching PSM-PROBE: {}",
            left["data"]["productVariants"]["nodes"]
                .as_array()
                .map_or(0, |a| a.len())
        ));
        let defs = all_definitions(probe, "cleanup_definitions");
        let scratch = defs
            .iter()
            .filter(|d| d["key"].as_str().is_some_and(|k| k.contains("psm_probe")))
            .count();
        probe.say(&format!(
            "  psm_probe definitions left: {scratch} (of {} definitions)",
            defs.len()
        ));
    }

    /// Live smoke of the real `ShopifyAdapter` (the adapter's own documents, not the probe's),
    /// with a `ConfirmedRun` from a scratch database. Everything created is on the ledger and
    /// deleted at the end.
    fn adapter_smoke(probe: &Probe) {
        use psm_core::{
            domain::publish::ConfirmedRun,
            ports::{clock::SystemClock, commerce::CommerceAdapter},
        };
        use psm_integrations::commerce::shopify::{PollConfig, ShopifyAdapter, ShopifyConfig};
        use std::sync::Arc;
        let dir = tempfile::tempdir().unwrap();
        let paths = psm_infra::paths::AppPaths::for_dir(dir.path());
        paths.create().unwrap();
        let store = psm_infra::sqlite::store::SqliteStore::open(&paths).unwrap();
        let conn = rusqlite::Connection::open(paths.db()).unwrap();
        conn.execute("INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('shopify',1,1,'probe smoke')", []).unwrap();
        let run = ConfirmedRun::from_persisted(&store, conn.last_insert_rowid()).unwrap();
        let adapter = ShopifyAdapter::new(
            ShopifyConfig {
                shop_domain: probe.shop.clone(),
                api_version: probe.version.clone(),
                client_id: probe.client_id.clone(),
                client_secret: probe.client_secret.clone(),
                condition_photos_key: "custom.damage_report".into(),
            },
            Arc::new(ReqwestTransport::new().unwrap()),
            Arc::new(SystemClock),
            PollConfig {
                interval_ms: 1000,
                attempts: 60,
                retries: 3,
                retry_wait_ms: 2000,
            },
        );
        let mut log = Vec::new();
        let mut note = |probe: &Probe, step: &str, outcome: String| {
            probe.say(&format!("  {step}: {outcome}"));
            log.push(json!({"step": step, "outcome": outcome}));
        };
        let sku = "PSM-PROBE-SMOKE-1";
        note(
            probe,
            "item_search before create",
            format!("{:?}", adapter.item_search(sku).map(|v| v.len())),
        );
        let id = adapter
            .item_create(&run, sku, Some(PROBE_BARCODE), "PSM-PROBE smoke product")
            .expect("create");
        probe.ledger_add("products", &id);
        let found = adapter.item_search(sku).unwrap();
        note(
            probe,
            "item_search after create",
            format!(
                "{} item(s), sku {:?}, barcode {:?}",
                found.len(),
                found.first().and_then(|i| i.sku.clone()),
                found.first().and_then(|i| i.barcode.clone())
            ),
        );
        let identity = adapter
            .item_set_identity(&run, &id, "PSM-PROBE-SMOKE-2", None)
            .unwrap();
        note(
            probe,
            "item_set_identity(SKU only)",
            format!("{identity:?}"),
        );
        let unchanged = adapter
            .item_set_identity(&run, &id, "PSM-PROBE-SMOKE-2", Some(PROBE_BARCODE))
            .unwrap();
        note(
            probe,
            "item_set_identity(no change)",
            format!("{unchanged:?}"),
        );
        let mut media = Vec::new();
        for (i, uid) in [
            "5e5e5e5e-0001-4000-8000-000000000001",
            "5e5e5e5e-0002-4000-8000-000000000002",
        ]
        .iter()
        .enumerate()
        {
            let bytes = fs::read(probe.dir.join(format!("probe{}.jpg", i + 2))).unwrap();
            let target = adapter
                .staged_uploads_create(&run, "psm-probe-smoke", uid, 0, bytes.len() as u64)
                .unwrap();
            adapter.upload(&run, &target, &bytes).unwrap();
            let m = adapter
                .product_create_media(&run, &id, &target.resource_url, &format!("smoke {}", i + 1))
                .unwrap();
            probe.ledger_add("media", &m);
            adapter.poll_media_ready(&m).unwrap();
            note(
                probe,
                &format!("media {} ready", i + 1),
                format!("{m} as {}", target.filename),
            );
            media.push(m);
        }
        let listed = adapter.list_media(&id).unwrap();
        note(
            probe,
            "list_media",
            format!(
                "{:?}",
                listed
                    .iter()
                    .map(|m| (m.position, m.filename.clone(), m.status))
                    .collect::<Vec<_>>()
            ),
        );
        let reversed: Vec<String> = media.iter().rev().cloned().collect();
        let job = adapter.product_reorder_media(&run, &id, &reversed).unwrap();
        adapter.poll_job(&job).unwrap();
        let order: Vec<String> = adapter
            .list_media(&id)
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();
        note(
            probe,
            "reorder (reverse) read-back equals D",
            format!("{}", order == reversed),
        );
        let bytes = fs::read(probe.dir.join("probe1.jpg")).unwrap();
        let target = adapter
            .staged_uploads_create(
                &run,
                "psm-probe-smoke",
                "5e5e5e5e-0003-4000-8000-000000000003",
                0,
                bytes.len() as u64,
            )
            .unwrap();
        adapter.upload(&run, &target, &bytes).unwrap();
        let file = adapter
            .file_create(&run, &target.resource_url, &target.filename)
            .unwrap();
        probe.ledger_add("files", &file);
        adapter.poll_file(&file).unwrap();
        note(
            probe,
            "file_create from an IMAGE staged target",
            format!("{file} READY"),
        );
        let schema = adapter.attributes_schema().unwrap();
        note(probe, "attributes_schema", format!("{} definitions, media_list: {:?}", schema.len(),
            schema.iter().filter(|(d, _)| d.core_type == psm_core::domain::attribute::CoreType::MediaList).map(|(d, _)| d.key.clone()).collect::<Vec<_>>()));
        let link = |key: &str| schema.iter().find(|(d, _)| d.key == key).unwrap().1.clone();
        let writes = vec![
            psm_core::domain::publish::WriteValue {
                key: "custom.damage_report".into(),
                link: link("custom.damage_report"),
                value: json!([file]),
            },
            psm_core::domain::publish::WriteValue {
                key: "beyo.fx_dimension".into(),
                link: link("beyo.fx_dimension"),
                value: json!({"unit": "CENTIMETERS", "value": "45.5"}),
            },
            psm_core::domain::publish::WriteValue {
                key: "beyo.fx_rating".into(),
                link: link("beyo.fx_rating"),
                value: json!({"max": "5", "min": "1", "value": "4.5"}),
            },
            psm_core::domain::publish::WriteValue {
                key: "beyo.fx_money".into(),
                link: link("beyo.fx_money"),
                value: json!({"amount": "12.5", "currency": "USD"}),
            },
            psm_core::domain::publish::WriteValue {
                key: "beyo.fx_color".into(),
                link: link("beyo.fx_color"),
                value: json!("#a1b2c3"),
            },
        ];
        note(
            probe,
            "attributes_write (damage report, dimension, rating, money, color)",
            format!("{:?}", adapter.attributes_write(&run, &id, &writes)),
        );
        let read = adapter.attributes_read(&id).unwrap();
        note(probe, "attributes_read", format!("{:?}", read));
        note(
            probe,
            "update_title",
            format!(
                "{:?}",
                adapter.update_title(&run, &id, "PSM-PROBE smoke product (renamed)")
            ),
        );
        note(
            probe,
            "update_description",
            format!(
                "{:?}",
                adapter.update_description(&run, &id, "a & b\n\nc < d \"q\"")
            ),
        );
        note(
            probe,
            "update_tags",
            format!(
                "{:?}",
                adapter.update_tags(&run, &id, &["a".into(), "b".into()])
            ),
        );
        let item = adapter.item_get(&id).unwrap();
        note(
            probe,
            "item_get",
            format!(
                "title {:?}, sku {:?}, barcode {:?}, media {}",
                item.title,
                item.sku,
                item.barcode,
                item.media.len()
            ),
        );
        note(
            probe,
            "product_delete_media",
            format!(
                "{:?}",
                adapter.product_delete_media(&run, &id, &[media[0].clone()])
            ),
        );
        note(
            probe,
            "product_delete_media again",
            format!(
                "{:?}",
                adapter.product_delete_media(&run, &id, &[media[0].clone()])
            ),
        );
        let left: Vec<String> = adapter
            .list_media(&id)
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();
        note(probe, "gallery after delete", format!("{left:?}"));
        probe.write_note("adapter_smoke_log", json!(log));
        cleanup(probe);
    }
}
