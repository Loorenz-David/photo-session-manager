//! R1·U1 automated rows (A01·C1–C3). The `[manual]` rows are evidence, not tests.

use crate::{Stub, files_containing, repo_root};
use psm_research_tool::{
    answer_key::{self, Bucket},
    budget::{Ledger, Limits, Refusal},
    recording::{Call, Caller, OutBody, Secrets, ToolError, sha256_hex},
};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;

#[test]
fn a01_c1a_naming_result_ignores_case_and_punctuation() {
    let key = vec!["Hans J. Wegner".to_owned()];
    assert!(!answer_key::is_naming("Wegner chair", &key));
    assert!(answer_key::is_naming("HANS J WEGNER - CH23", &key));
}

#[test]
fn a01_c1b_own_listing_is_the_shop_path_ending_in_the_handle() {
    let shop = vec!["shop.example".to_owned()];
    let own = |url: &str| answer_key::is_own_listing(url, "oak-table", &shop);
    assert!(own("https://shop.example/en/products/oak-table"));
    assert!(own("https://shop.example/products/oak-table?x=1"));
    assert!(!own("https://shop.example/products/oak-table-2"));
    assert!(!own("https://other.example/products/oak-table"));
}

#[test]
fn a01_c1c_every_correction_is_scored_in_the_corrected_bucket_only() {
    let root = repo_root();
    let pieces = answer_key::load_pieces(&root.join(answer_key::TEST_FILE)).unwrap();
    let corrections =
        answer_key::load_corrections(&root.join(answer_key::CORRECTIONS_FILE)).unwrap();
    let entries = answer_key::key_entries(&pieces, &corrections);
    let corrected: Vec<_> = entries
        .iter()
        .filter(|e| e.bucket == Bucket::Corrected)
        .collect();
    assert_eq!(corrected.len(), corrections.len());
    assert_eq!(corrections.len(), 6);
    for c in &corrections {
        let same = |e: &&answer_key::KeyEntry| e.n == c.n && e.field == c.field;
        let found: Vec<_> = entries.iter().filter(same).collect();
        assert_eq!(found.len(), 1, "piece {} field {}", c.n, c.field);
        assert_eq!(found[0].bucket, Bucket::Corrected);
        assert_eq!(found[0].value, c.corrected_value);
        assert!(
            !entries
                .iter()
                .any(|e| e.n == c.n && e.field == c.field && e.bucket == Bucket::OwnerVerified)
        );
    }
}

fn probe_call(dir: &std::path::Path, label: &str, url: &str, worst: i64) -> Call {
    let mut call = Call::new(label, dir, "openai", "openai", "GET", url);
    call.worst_case_micro = worst;
    call
}

#[test]
fn a01_c2a_budget_refuses_a_call_that_would_reach_the_ceiling_before_sending() {
    let out = tempfile::tempdir().unwrap();
    let stub = Stub::default();
    let limits = Limits {
        ceiling_micro: 1_000_000,
        caps: BTreeMap::new(),
    };
    let caller = Caller::new(&stub, Secrets::default(), Ledger::new(970_000), limits);
    let result = caller.call(probe_call(
        out.path(),
        "model",
        "https://api.example/v1",
        40_000,
    ));
    match result {
        Err(ToolError::Refused(Refusal::Ceiling {
            spent,
            worst_case,
            ceiling,
        })) => assert_eq!((spent, worst_case, ceiling), (970_000, 40_000, 1_000_000)),
        other => panic!("expected a ceiling refusal, got {other:?}"),
    }
    assert!(stub.received.borrow().is_empty());
}

#[test]
fn a01_c2b_budget_refuses_when_a_call_cap_is_reached_naming_the_cap() {
    let out = tempfile::tempdir().unwrap();
    let stub = Stub::default();
    let mut ledger = Ledger::new(0);
    ledger.counts.insert("openai".into(), 80);
    let caller = Caller::new(&stub, Secrets::default(), ledger, Limits::u1(15_000_000));
    let err = caller
        .call(probe_call(out.path(), "model", "https://api.example/v1", 1))
        .unwrap_err();
    match &err {
        ToolError::Refused(Refusal::Cap { cap, limit }) => {
            assert_eq!((cap.as_str(), *limit), ("openai", 80))
        }
        other => panic!("expected a cap refusal, got {other:?}"),
    }
    assert!(err.to_string().contains("call cap openai = 80"));
    assert!(stub.received.borrow().is_empty());
}

#[test]
fn a01_c2c_bench_search_without_ceiling_exits_non_zero_before_any_request() {
    let out = tempfile::tempdir().unwrap();
    let stub = Stub::default();
    let args: Vec<String> = ["bench-search", "--out", out.path().to_str().unwrap()]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_ne!(psm_research_tool::run(&args, &stub), 0);
    assert!(stub.received.borrow().is_empty());
    // The binary itself, too.
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_psm-research"))
        .args(&args)
        .env_remove("OPENAI_API_KEY")
        .env_remove("SERPAPI_API_KEY")
        .env_remove("GOOGLE_VISION_API_KEY")
        .output()
        .unwrap();
    assert!(!status.status.success());
    assert!(String::from_utf8_lossy(&status.stderr).contains("--ceiling-usd is required"));
}

const KEY: &str = "sk-TEST/ab+cd=ef0123456789";

#[test]
fn a01_c3a_a_key_value_never_reaches_the_out_folder() {
    let out = tempfile::tempdir().unwrap();
    let encoded = "sk-TEST%2Fab%2Bcd%3Def0123456789";
    let stub = Stub::default();
    let caller = Caller::new(
        &stub,
        Secrets::new([KEY.to_owned()]),
        Ledger::new(0),
        Limits::u1(15_000_000),
    );
    // Plain: in the URL query, a header and the response body.
    stub.reply(200, &format!("{{\"echo\":\"{KEY}\"}}"));
    let mut plain = probe_call(
        out.path(),
        "plain",
        &format!("https://api.example/v1?key={KEY}&q=1"),
        0,
    );
    plain.headers = vec![("x-goog-api-key".into(), KEY.into())];
    let err = caller.call(plain).unwrap_err();
    match &err {
        ToolError::SecretInRecording { call_id } => assert!(call_id.starts_with("plain-")),
        other => panic!("expected a secret refusal, got {other:?}"),
    }
    assert!(err.to_string().contains("call plain-"));
    assert!(!err.to_string().contains(KEY));
    // URL-encoded.
    stub.reply(200, &format!("{{\"echo\":\"{encoded}\"}}"));
    let mut enc = probe_call(
        out.path(),
        "encoded",
        &format!("https://api.example/v1?api_key={encoded}"),
        0,
    );
    enc.headers = vec![("authorization".into(), format!("Bearer {encoded}"))];
    let err = caller.call(enc).unwrap_err();
    assert!(err.to_string().contains("call encoded-"));
    // Declared parts only: written, and they read <redacted>.
    stub.reply(200, "{\"ok\":true}");
    let mut declared = probe_call(
        out.path(),
        "declared",
        &format!("https://api.example/v1?key={KEY}&q=1"),
        0,
    );
    declared.headers = vec![("x-goog-api-key".into(), KEY.into())];
    let recorded = caller.call(declared).unwrap();
    let text = std::fs::read_to_string(&recorded.file).unwrap();
    let json: Json = serde_json::from_str(&text).unwrap();
    assert_eq!(
        json["request"]["url"],
        "https://api.example/v1?key=<redacted>&q=1"
    );
    assert_eq!(json["request"]["headers"]["x-goog-api-key"], "<redacted>");
    assert!(files_containing(out.path(), KEY).is_empty());
    assert!(files_containing(out.path(), encoded).is_empty());
    assert_eq!(stub.received.borrow().len(), 3);
}

#[test]
fn a01_c3b_two_images_are_recorded_by_sha256_and_stored_once_each() {
    let out = tempfile::tempdir().unwrap();
    let stub = Stub::default();
    let caller = Caller::new(
        &stub,
        Secrets::default(),
        Ledger::new(0),
        Limits::u1(15_000_000),
    );
    let (first, second) = (
        b"first image bytes".to_vec(),
        b"second image bytes".to_vec(),
    );
    let mut call = Call::new(
        "two-images",
        out.path(),
        "openai",
        "openai",
        "POST",
        "https://api.example/v1/responses",
    );
    call.body = OutBody::Json(json!({"input": [
        {"type": "input_image", "image_url": "data:image/jpeg;base64,@@IMAGE:0@@"},
        {"type": "input_image", "image_url": "data:image/jpeg;base64,@@IMAGE:1@@"},
    ]}));
    call.images = vec![first.clone(), second.clone()];
    let recorded = caller.call(call).unwrap();
    let json: Json =
        serde_json::from_str(&std::fs::read_to_string(&recorded.file).unwrap()).unwrap();
    let items = json["request"]["body"]["input"].as_array().unwrap();
    assert_eq!(
        items[0]["image_url"],
        json!({"image_sha256": sha256_hex(&first)})
    );
    assert_eq!(
        items[1]["image_url"],
        json!({"image_sha256": sha256_hex(&second)})
    );
    let mut names: Vec<String> = std::fs::read_dir(out.path().join("images"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut expected = vec![
        format!("{}.jpg", sha256_hex(&first)),
        format!("{}.jpg", sha256_hex(&second)),
    ];
    expected.sort();
    assert_eq!(names, expected);
    assert_eq!(
        std::fs::read(
            out.path()
                .join("images")
                .join(format!("{}.jpg", sha256_hex(&first)))
        )
        .unwrap(),
        first
    );
}
