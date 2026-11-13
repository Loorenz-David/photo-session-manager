//! R1·U2 automated rows (A02·C1, C2). The `[manual]` rows are evidence, not tests.

use crate::{Stub, repo_root};
use psm_research_tool::{
    Args, Ctx,
    answer_key::{self, Bucket},
    bench_models::{Planned, execute},
    budget::{Ledger, Limits},
    openai::ModelCall,
    recording::{Caller, Secrets, call_files},
    score::{self, KeyKind, Outcome},
};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;

fn choices(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn a02_c1a_choice_answers_are_mapped_by_name_key() {
    let kind = KeyKind::Choice(choices(&["Teak", "Oak", "Walnut"]));
    let key = json!("Teak");
    assert_eq!(
        score::score(&kind, &key, Some(&json!("Teak"))),
        Outcome::Correct
    );
    assert_eq!(
        score::score(&kind, &key, Some(&json!("teak"))),
        Outcome::Correct
    );
    assert_eq!(
        score::score(&kind, &key, Some(&json!("Oak"))),
        Outcome::Wrong
    );
    assert_eq!(score::score(&kind, &key, None), Outcome::NotDetermined);
}

#[test]
fn a02_c1b_multi_choice_is_correct_only_for_the_same_set() {
    let kind = KeyKind::MultiChoice(choices(&["Teak", "Oak", "Walnut"]));
    let key = json!(["Teak", "Oak"]);
    assert_eq!(
        score::score(&kind, &key, Some(&json!(["Oak", "Teak"]))),
        Outcome::Correct
    );
    assert_eq!(
        score::score(&kind, &key, Some(&json!(["Teak"]))),
        Outcome::Wrong
    );
}

#[test]
fn a02_c1c_text_is_compared_by_name_key_without_aliases() {
    let key = json!("Hans J. Wegner");
    assert_eq!(
        score::score(&KeyKind::Text, &key, Some(&json!("hans j wegner"))),
        Outcome::Correct
    );
    assert_eq!(
        score::score(&KeyKind::Text, &key, Some(&json!("Hans Wegner"))),
        Outcome::Wrong
    );
}

#[test]
fn a02_c1d_a_corrected_field_is_scored_against_the_corrected_value_only() {
    let root = repo_root();
    let pieces = answer_key::load_pieces(&root.join(answer_key::TEST_FILE)).unwrap();
    let corrections =
        answer_key::load_corrections(&root.join(answer_key::CORRECTIONS_FILE)).unwrap();
    let entries = answer_key::key_entries(&pieces, &corrections);
    // Piece 39: the file says Beech, the corrections file says Cherry.
    let kind = KeyKind::MultiChoice(choices(&["Teak", "Oak", "Cherry", "Beech"]));
    assert_eq!(
        score::score_field(&entries, 39, "wood_type", &kind, Some(&json!(["Cherry"]))),
        Some((Bucket::Corrected, Outcome::Correct))
    );
    assert_eq!(
        score::score_field(&entries, 39, "wood_type", &kind, Some(&json!(["Beech"]))),
        Some((Bucket::Corrected, Outcome::Wrong))
    );
    assert!(
        !entries
            .iter()
            .any(|e| e.n == 39 && e.field == "wood_type" && e.bucket == Bucket::OwnerVerified)
    );
}

#[test]
fn a02_c1e_a_page_that_does_not_name_the_key_states_nothing() {
    assert_eq!(
        score::score_read(false, "Hans J. Wegner", None),
        Outcome::Correct
    );
    assert_eq!(
        score::score_read(false, "Hans J. Wegner", Some("Arne Jacobsen")),
        Outcome::Wrong
    );
}

fn planned(out: &std::path::Path, i: usize) -> Planned {
    let mut mc = ModelCall::new("gpt-test", &format!("piece {i}"));
    mc.max_output_tokens = 100;
    Planned {
        label: format!("observe-cover-{i}"),
        dir: out.join(format!("{i}-piece")).join("run-bench2"),
        mc,
        meta: json!({"stage": "observe", "n": i, "unit": "cover"}),
    }
}

fn ctx<'a>(stub: &'a Stub, out: &std::path::Path, caps: u32) -> Ctx<'a> {
    let caller = Caller::new(
        stub,
        Secrets::default(),
        Ledger::from_recordings(out),
        Limits {
            ceiling_micro: 35_000_000,
            caps: BTreeMap::from([("openai".to_owned(), caps)]),
        },
    );
    Ctx {
        args: Args::default(),
        out: out.to_path_buf(),
        caller,
        seam: stub,
    }
}

#[test]
fn a02_c2a_a_stopped_run_resumes_without_repeating_calls_or_spend() {
    let out = tempfile::tempdir().unwrap();
    std::fs::write(
        out.path().join("prices.json"),
        json!({"models": {"gpt-test": {"input_token": "1.00", "cached_input_token": "0.10", "output_token": "5.00"}},
               "tools": {"web_search_call": "10000.00"}})
        .to_string(),
    )
    .unwrap();
    let reply =
        r#"{"status":"completed","output":[],"usage":{"input_tokens":1000,"output_tokens":100}}"#;
    // First run: stopped after 3 of 5 calls (the call cap stands in for the interruption).
    let first = Stub::default();
    (0..5).for_each(|_| first.reply(200, reply));
    let run1 = ctx(&first, out.path(), 3);
    assert!(
        execute(
            &run1,
            (1..=5).map(|i| planned(out.path(), i)).collect(),
            "k"
        )
        .is_err()
    );
    assert_eq!(first.received.borrow().len(), 3);
    // Second run, started again from the recordings.
    let second = Stub::default();
    (0..5).for_each(|_| second.reply(200, reply));
    let run2 = ctx(&second, out.path(), 700);
    execute(
        &run2,
        (1..=5).map(|i| planned(out.path(), i)).collect(),
        "k",
    )
    .unwrap();
    assert_eq!(second.received.borrow().len(), 2);
    assert_eq!(call_files(out.path()).len(), 5);
    // 1,000 input tokens at USD 1/M + 100 output tokens at USD 5/M = 1,500 micro-USD per call.
    let ledger = Ledger::from_recordings(out.path());
    assert_eq!(ledger.spent_micro, 5 * 1_500);
    assert_eq!(ledger.counts.get("openai"), Some(&5));
    assert_eq!(run2.caller.ledger.borrow().spent_micro, 5 * 1_500);
    let costs: Vec<i64> = call_files(out.path())
        .iter()
        .map(|f| serde_json::from_str::<Json>(&std::fs::read_to_string(f).unwrap()).unwrap()["cost_micro_usd"].as_i64().unwrap())
        .collect();
    assert_eq!(costs, vec![1_500; 5]);
}
