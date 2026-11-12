//! R4·U2: `CallGateway::call` on the harness with a `running` run row; outcomes are the rows of
//! `research_calls`, `research_outbound`, `research_runs` and the fakes' request logs.

use std::sync::{Arc, Mutex};

use psm_infra::sqlite::store::SqliteStore;
use psm_research::{
    domain::{ErrorClass, RunState, Stage},
    fakes::{self, FakeModel, ModelScript, PageScript},
    gateway::{Answer, CallRequest, GatewayOutcome},
    images::ResearchImage,
    ports::ModelRequest,
    rules::cost::call_cost,
    store as rstore,
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::{Value as Json, json};

use crate::harness::{Harness, MODEL_KEY};

fn model(stage: Stage, unit: &str, images: Vec<ResearchImage>) -> CallRequest {
    CallRequest::Model(ModelRequest {
        stage,
        unit_key: unit.to_owned(),
        model: "m1".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: "look".into(),
        input: json!({"keys": ["custom.wood"]}),
        images,
        listing_images: Vec::new(),
    })
}

/// A harness with a running run and two rendered main photos.
fn two_photos() -> (Harness, i64, Vec<ResearchImage>) {
    let h = Harness::new();
    let a = h.add_main_photo_with_preview(h.session_id);
    let b = h.add_main_photo_with_preview(h.session_id);
    let run = h.running_run(&h.config());
    let images = vec![
        h.images.render(a, run, 1024).unwrap(),
        h.images.render(b, run, 1024).unwrap(),
    ];
    (h, run, images)
}

fn delete_by_sql(store: &SqliteStore, photo_id: i64) {
    store
        .research_transaction(|tx| {
            tx.conn()
                .execute(
                    "UPDATE photos SET classification='deleted' WHERE id=?1",
                    [photo_id],
                )
                .map(|_| ())
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))
        })
        .unwrap();
}

/// Call states and the outbound count seen from inside the hook.
type Seen = (Vec<String>, usize);

#[test]
fn a11_c1a_record_and_outbound_committed_before_send() {
    let (h, run, images) = two_photos();
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::default();
    let (store, out) = (h.store().clone(), seen.clone());
    h.model.before_call(move |_| {
        let states: Vec<String> = store
            .research_transaction(|tx| {
                Ok(rstore::calls(tx.conn(), Some(run))?
                    .into_iter()
                    .map(|c| c.state)
                    .collect())
            })
            .unwrap();
        let outbound = store
            .research_transaction(|tx| rstore::outbound(tx.conn(), Some(run)))
            .unwrap()
            .len();
        out.lock().unwrap().push((states, outbound));
    });
    h.gateway()
        .call(
            run,
            Stage::Observe,
            "photo:x",
            model(Stage::Observe, "photo:x", images),
        )
        .unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![(vec!["sent".to_owned()], 2)]);
}

#[test]
fn a11_c1b_outbound_rows_equal_images_received() {
    let (h, run, images) = two_photos();
    h.gateway()
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    let received: Vec<String> = h.model.requests()[0]
        .images
        .iter()
        .map(|(sha, _)| sha.clone())
        .collect();
    let outbound: Vec<String> = h.outbound(run).into_iter().map(|o| o.sha256).collect();
    assert_eq!(outbound, received);
    assert_eq!(
        received,
        images
            .iter()
            .map(|i| i.sha256().to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn a11_c2a_photo_deleted_before_the_gate_is_dropped() {
    let (h, run, images) = two_photos();
    let gone = images[1].photo_id();
    h.app.review().delete(gone).unwrap();
    let called = h
        .gateway()
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    assert_eq!(called.dropped, vec![gone]);
    assert_eq!(
        h.model.requests()[0].images,
        vec![(images[0].sha256().to_owned(), 1024)]
    );
    let outbound = h.outbound(run);
    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0].photo_id, images[0].photo_id());
}

#[test]
fn a11_c2b_photo_deleted_in_flight_is_excluded_from_the_next_call() {
    let (h, run, images) = two_photos();
    let gone = images[1].photo_id();
    let store = h.store().clone();
    let mut fired = false;
    h.model.before_call(move |_| {
        if !fired {
            fired = true;
            delete_by_sql(&store, gone);
        }
    });
    let gateway = h.gateway();
    gateway
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    gateway
        .call(
            run,
            Stage::Labels,
            "u",
            model(Stage::Labels, "u", images.clone()),
        )
        .unwrap();
    let calls = h.calls(run);
    let outbound = h.outbound(run);
    let of = |call: i64| -> Vec<i64> {
        outbound
            .iter()
            .filter(|o| o.call_id == call)
            .map(|o| o.photo_id)
            .collect()
    };
    assert_eq!(of(calls[0].id), vec![images[0].photo_id(), gone]);
    assert_eq!(of(calls[1].id), vec![images[0].photo_id()]);
    assert_eq!(h.model.requests()[1].images.len(), 1);
}

#[test]
fn a11_c2c_nothing_left_sends_nothing() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.running_run(&h.config());
    let image = h.images.render(photo, run, 1024).unwrap();
    h.app.review().delete(photo).unwrap();
    let called = h
        .gateway()
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", vec![image]),
        )
        .unwrap();
    assert_eq!(called.dropped, vec![photo]);
    assert!(h.model.requests().is_empty());
    assert!(h.calls(run).is_empty());
    assert!(h.outbound(run).is_empty());
}

#[test]
fn a11_c3a_answered_record() {
    let (h, run, images) = two_photos();
    h.model.script(
        Stage::Observe,
        "u",
        ModelScript {
            output: json!({"reads": {}}),
            usage: Some(vec![
                ("input_token".into(), 1200),
                ("output_token".into(), 30),
            ]),
            response: Some(json!({"id": "resp-1", "output": "ok"})),
        },
    );
    let called = h
        .gateway()
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    assert!(matches!(called.outcome, GatewayOutcome::Answered(_)));
    let call = &h.calls(run)[0];
    assert_eq!(call.state, "answered");
    let request: Json = serde_json::from_str(&call.request).unwrap();
    assert_eq!(
        request["images"],
        json!([
            {"image_sha256": images[0].sha256()},
            {"image_sha256": images[1].sha256()}
        ])
    );
    let response: Json = serde_json::from_str(call.response.as_deref().unwrap()).unwrap();
    assert_eq!(response["body"], json!({"id": "resp-1", "output": "ok"}));
    let usage: Json = serde_json::from_str(call.usage.as_deref().unwrap()).unwrap();
    assert_eq!(usage, json!({"input_token": 1200, "output_token": 30}));
    assert_eq!(call.cost_micro_usd, 1260);
    assert_eq!(call.started_at, h.now());
    assert_eq!(call.finished_at, Some(h.now()));
    assert_eq!(call.duration_ms, Some(0));
}

#[test]
fn a11_c3b_listing_images_are_stored_not_outbound() {
    let h = Harness::new();
    let run = h.running_run(&h.config());
    let first = SyntheticJpeg::rgb(40, 30, [1, 2, 3]);
    let second = SyntheticJpeg::rgb(40, 30, [200, 100, 50]);
    h.reader.set_page(
        "https://e1.example/p/1",
        PageScript::page("Sideboard", vec![first.clone(), second.clone()]),
    );
    let called = h
        .gateway()
        .call(
            run,
            Stage::Read,
            "https://e1.example/p/1",
            CallRequest::Read {
                url: "https://e1.example/p/1".into(),
            },
        )
        .unwrap();
    let GatewayOutcome::Answered(Answer::Page { image_sha256, .. }) = called.outcome else {
        panic!("expected a page");
    };
    let files = h.files();
    let expected = vec![
        psm_research::files::sha256_hex(&first),
        psm_research::files::sha256_hex(&second),
    ];
    assert_eq!(image_sha256, expected);
    for sha in &expected {
        assert!(files.exists(sha));
    }
    let call = &h.calls(run)[0];
    assert_eq!(
        psm_research::gateway::CallGateway::page_images(call.response.as_deref().unwrap()),
        expected
    );
    assert!(h.outbound(run).is_empty());
}

fn url_encoded(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// A fault whose text carries the model key, then a reply whose body carries it URL-encoded.
fn leaky_call(h: &Harness, run: i64, images: Vec<ResearchImage>) {
    h.model.fault(fakes::transient(&format!(
        "upstream said: bad key {MODEL_KEY}"
    )));
    h.model.script(
        Stage::Observe,
        "u",
        ModelScript {
            output: json!({}),
            usage: None,
            response: Some(
                json!({"echo": format!("https://x.example/?key={}", url_encoded(MODEL_KEY))}),
            ),
        },
    );
    h.gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
}

#[test]
fn a11_c4a_secrets_are_redacted_in_the_record() {
    let (h, run, images) = two_photos();
    leaky_call(&h, run, images);
    let calls = h.calls(run);
    assert_eq!(calls.len(), 2);
    assert!(
        calls[0]
            .error_text
            .as_deref()
            .unwrap()
            .contains("<redacted>")
    );
    assert!(calls[1].response.as_deref().unwrap().contains("<redacted>"));
    assert!(calls.iter().all(|c| c.redaction_hit));
    let dump: String = h.query(|c| {
        let mut stmt = c
            .prepare("SELECT * FROM research_calls UNION ALL SELECT id,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,error,0 FROM research_runs")
            .unwrap();
        let n = stmt.column_count();
        stmt.query_map([], |r| {
            Ok((0..n)
                .map(|i| match r.get_ref(i).unwrap() {
                    rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                    other => format!("{other:?}"),
                })
                .collect::<Vec<_>>()
                .join("|"))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join("\n")
    });
    assert!(!dump.contains(MODEL_KEY));
    assert!(!dump.contains(&url_encoded(MODEL_KEY)));
    assert!(!dump.contains(&url_encoded(MODEL_KEY).to_lowercase()));
}

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Lines {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a11_c4b_no_log_line_carries_a_secret() {
    psm_testkit::isolation::in_own_process(
        module_path!(),
        "a11_c4b_no_log_line_carries_a_secret",
        || {
            let lines = Lines::default();
            let writer = lines.clone();
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .with_max_level(tracing::Level::TRACE)
                .with_writer(move || writer.clone())
                .finish();
            tracing::subscriber::with_default(subscriber, || {
                let (h, run, images) = two_photos();
                leaky_call(&h, run, images);
            });
            let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
            assert!(!text.contains(MODEL_KEY));
            assert!(!text.contains(&url_encoded(MODEL_KEY)));
            let errors: Vec<&str> = text
                .lines()
                .filter(|l| l.contains("ERROR") && l.contains("redacted"))
                .collect();
            assert_eq!(errors.len(), 1, "{text}");
        },
    );
}

#[test]
fn a11_c5a_attempts_wait_one_then_two_seconds() {
    let (h, run, images) = two_photos();
    h.model.fault(fakes::transient("timed out"));
    h.model.fault(fakes::transient("timed out"));
    let t = h.now();
    let called = h
        .gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    assert!(matches!(called.outcome, GatewayOutcome::Answered(_)));
    let calls = h.calls(run);
    assert_eq!(
        calls.iter().map(|c| c.attempt).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        calls.iter().map(|c| c.started_at - t).collect::<Vec<_>>(),
        vec![0, 1_000, 3_000]
    );
}

#[test]
fn a11_c5b_attempts_run_out_and_the_run_fails() {
    let (h, run, images) = two_photos();
    for _ in 0..3 {
        h.model.fault(fakes::transient("timed out"));
    }
    let called = h
        .gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    assert!(matches!(
        called.outcome,
        GatewayOutcome::RunFailed {
            stage: Stage::Observe,
            class: ErrorClass::Transient,
            ..
        }
    ));
    let row = h.run_row(run);
    assert_eq!(row.state, RunState::Failed);
    assert_eq!(row.error.as_deref(), Some("observe failed: timed out"));
    assert_eq!(h.calls(run).len(), 3);
}

#[test]
fn a11_c5c_rejected_key_fails_without_retry() {
    let (h, run, images) = two_photos();
    h.model.fault(fakes::status(401, "invalid api key"));
    h.gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    assert_eq!(h.calls(run).len(), 1);
    let row = h.run_row(run);
    assert_eq!(row.state, RunState::Failed);
    assert_eq!(
        row.error.as_deref(),
        Some(format!("key rejected by {}", FakeModel::ID).as_str())
    );
    assert_eq!(h.model.requests().len(), 1);
}

#[test]
fn a11_c5d_refusal_is_not_determined() {
    let (h, run, images) = two_photos();
    h.model.fault(fakes::refusal("I can't help with that"));
    let called = h
        .gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    assert_eq!(
        called.outcome,
        GatewayOutcome::NotDetermined("I can't help with that".into())
    );
    assert_eq!(h.model.requests().len(), 1);
    assert_eq!(h.run_row(run).state, RunState::Running);
}

#[test]
fn a11_c5e_offline_is_not_sent_and_counts_no_attempt() {
    let (h, run, images) = two_photos();
    h.model.fault(fakes::offline());
    let gateway = h.gateway();
    let called = gateway
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    assert_eq!(called.outcome, GatewayOutcome::HoldOffline);
    let first = &h.calls(run)[0];
    assert_eq!(
        (first.state.as_str(), first.cost_micro_usd),
        ("not_sent", 0)
    );
    gateway
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    let calls = h.calls(run);
    assert_eq!(calls[1].attempt, 1);
    assert_eq!(calls[1].state, "answered");
}

#[test]
fn a11_c6a_cost_from_usage_and_price_snapshot() {
    let (h, run, images) = two_photos();
    let usage = vec![
        ("input_token".to_owned(), 2_345),
        ("output_token".to_owned(), 101),
    ];
    h.model.script(
        Stage::Observe,
        "u",
        ModelScript {
            output: json!({}),
            usage: Some(usage.clone()),
            response: None,
        },
    );
    let observed: Arc<Mutex<Vec<(i64, i64)>>> = Arc::default();
    let gateway = h.gateway();
    gateway
        .call(
            run,
            Stage::Observe,
            "u",
            model(Stage::Observe, "u", images.clone()),
        )
        .unwrap();
    // A second call's hook sees the run's cost equal to the sum of the committed calls.
    let (store, out) = (h.store().clone(), observed.clone());
    h.model.before_call(move |_| {
        let pair = store
            .research_transaction(|tx| {
                let run_cost = rstore::run(tx.conn(), run)?.unwrap().cost_micro_usd;
                let calls: i64 = rstore::calls(tx.conn(), Some(run))?
                    .iter()
                    .map(|c| c.cost_micro_usd)
                    .sum();
                Ok((run_cost, calls))
            })
            .unwrap();
        out.lock().unwrap().push(pair);
    });
    gateway
        .call(run, Stage::Labels, "u", model(Stage::Labels, "u", images))
        .unwrap();
    let expected = call_cost(
        Some(&usage),
        true,
        FakeModel::ID,
        "m1",
        &h.config().price_table(),
    );
    assert_eq!(expected, 2_547);
    assert_eq!(h.calls(run)[0].cost_micro_usd, expected);
    assert_eq!(*observed.lock().unwrap(), vec![(expected, expected)]);
}

#[test]
fn a11_c6b_timeout_after_send_is_charged_the_fallback() {
    let (h, run, images) = two_photos();
    h.model.fault(fakes::transient("timed out"));
    h.gateway()
        .call(run, Stage::Observe, "u", model(Stage::Observe, "u", images))
        .unwrap();
    let calls = h.calls(run);
    assert_eq!(
        (calls[0].state.as_str(), calls[0].cost_micro_usd),
        ("failed", 1_000)
    );
    assert_eq!(
        h.run_row(run).cost_micro_usd,
        1_000 + calls[1].cost_micro_usd
    );
}

#[test]
fn a11_c7a_reserve_reaches_the_cap_skips_to_decide() {
    let h = Harness::new();
    let config = h.config();
    let run = h.running_run(&config);
    // cap 500 000; reserve = the decide model's fallback, 1 000.
    h.set_run_cost(run, 499_000);
    let called = h
        .gateway()
        .call(
            run,
            Stage::Read,
            "https://e1.example/p",
            CallRequest::Read {
                url: "https://e1.example/p".into(),
            },
        )
        .unwrap();
    assert_eq!(called.outcome, GatewayOutcome::SkipToDecide);
    assert!(h.calls(run).is_empty());
    assert!(h.reader.requests().is_empty());
}

#[test]
fn a11_c7b_cap_reached_ends_at_cap() {
    let h = Harness::new();
    let run = h.running_run(&h.config());
    h.set_run_cost(run, 500_000);
    let called = h
        .gateway()
        .call(
            run,
            Stage::Decide,
            "decide",
            model(Stage::Decide, "decide", vec![]),
        )
        .unwrap();
    assert_eq!(called.outcome, GatewayOutcome::EndAtCap);
    assert!(h.calls(run).is_empty());
    assert!(h.model.requests().is_empty());
}
