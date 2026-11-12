//! The research harness: a `TestApp` with a real SQLite store, one item of type Sideboard and the
//! definitions the write-path rows need. Later units extend it.

use psm_core::domain::{
    attribute::{AiConfidence, AttributeGroup, CoreType, Definition, Preset, PresetField, Value},
    research::{Feedback, FeedbackAction},
};
use psm_research::writes::{self, WriteOutcome};
use psm_testkit::TestApp;
use serde_json::{Value as Json, json};

pub const INTEGRATION: &str = "shop";
pub const T0: i64 = 1_761_350_400_000;

/// Text field in the Details step.
pub const MAKER: &str = "custom.maker";
/// Choice field `[Oak, Teak]` in the Details step.
pub const WOOD: &str = "custom.wood";
/// A Details definition left out of the item type's layout.
pub const OUTSIDE: &str = "test.outside";
/// A text definition of the Product step.
pub const SUBTITLE: &str = "custom.subtitle";
pub const MEDIA: &str = "test.media";
pub const REFERENCE: &str = "test.reference";
pub const RICH: &str = "test.rich";
pub const MONEY: &str = "test.money";
pub const RATING: &str = "test.rating";
/// A text definition with `writable = false`.
pub const LOCKED: &str = "test.locked";

fn def(key: &str, core_type: CoreType, group: AttributeGroup, choices: &[&str]) -> Definition {
    Definition {
        integration: INTEGRATION.into(),
        key: key.into(),
        label: key.into(),
        core_type,
        choices: choices.iter().map(|c| c.to_string()).collect(),
        validations: json!({}),
        group,
        description: None,
        writable: key != LOCKED,
    }
}

fn definitions() -> Vec<Definition> {
    use AttributeGroup::{Details, Product};
    use CoreType as T;
    vec![
        def(MAKER, T::Text, Details, &[]),
        def(WOOD, T::Choice, Details, &["Oak", "Teak"]),
        def(OUTSIDE, T::Text, Details, &[]),
        def(SUBTITLE, T::Text, Product, &[]),
        def(MEDIA, T::MediaList, Details, &[]),
        def(REFERENCE, T::Reference, Details, &[]),
        def(RICH, T::RichText, Details, &[]),
        def(MONEY, T::Money, Details, &[]),
        def(RATING, T::Rating, Details, &[]),
        def(LOCKED, T::Text, Details, &[]),
    ]
}

pub struct Harness {
    pub app: TestApp,
    pub item_id: i64,
    pub session_id: i64,
}

impl Harness {
    pub fn new() -> Self {
        let app = TestApp::new().unwrap();
        app.clock.advance(T0);
        let attributes = app.attributes(Some(INTEGRATION));
        attributes
            .import_definitions(INTEGRATION, definitions())
            .unwrap();
        let details: Vec<PresetField> = definitions()
            .into_iter()
            .filter(|d| d.group == AttributeGroup::Details && d.key != OUTSIDE)
            .map(|d| PresetField {
                key: d.key,
                default: None,
            })
            .collect();
        attributes
            .presets_set(Preset {
                item_type: "Sideboard".into(),
                group: AttributeGroup::Details,
                layout: details,
            })
            .unwrap();
        let (item, session) = app
            .items()
            .create_local("R3-1", "Oak sideboard", "Sideboard", None)
            .unwrap();
        Harness {
            item_id: item.id,
            session_id: session.session.id,
            app,
        }
    }

    pub fn now(&self) -> i64 {
        use psm_core::ports::clock::Clock;
        self.app.clock.now_ms()
    }

    pub fn advance(&self, ms: i64) {
        self.app.clock.advance(ms);
    }

    pub fn definition(&self, key: &str) -> Definition {
        definitions().into_iter().find(|d| d.key == key).unwrap()
    }

    pub fn suggest_with(
        &self,
        key: &str,
        value: Json,
        confidence: AiConfidence,
        reason: Option<&str>,
        sources: &[String],
    ) -> WriteOutcome {
        writes::suggest(
            self.app.store(),
            self.app.clock.as_ref(),
            Some(INTEGRATION),
            self.item_id,
            key,
            &value,
            confidence,
            reason,
            sources,
        )
        .unwrap()
    }

    pub fn suggest(&self, key: &str, value: Json) -> WriteOutcome {
        self.suggest_with(
            key,
            value,
            AiConfidence::Medium,
            Some("seen on two sites"),
            &[],
        )
    }

    pub fn lower(&self, key: &str, reason: &str) -> WriteOutcome {
        writes::lower(
            self.app.store(),
            self.app.clock.as_ref(),
            self.item_id,
            key,
            reason,
        )
        .unwrap()
    }

    /// Stores a row straight through the store.
    pub fn put(&self, row: &Value) {
        self.app
            .store()
            .transaction(&mut |tx| tx.put_attribute_value(row))
            .unwrap();
    }

    /// The stored row of `key`, cleared rows included.
    pub fn row(&self, key: &str) -> Option<Value> {
        let mut out = None;
        self.app
            .store()
            .transaction(&mut |tx| {
                out = tx
                    .attribute_values(self.item_id)?
                    .into_iter()
                    .find(|v| v.key == key);
                Ok(())
            })
            .unwrap();
        out
    }

    /// Inserts a feedback row for `key` with the given action and suggested value.
    pub fn feedback(&self, key: &str, action: FeedbackAction, suggested: Json) {
        let feedback = Feedback {
            id: 0,
            item_id: self.item_id,
            key: key.into(),
            suggested_value: suggested.to_string(),
            confidence: AiConfidence::Medium,
            final_value: None,
            action,
            at: self.now(),
            step_token: None,
        };
        self.app
            .store()
            .transaction(&mut |tx| tx.insert_feedback(&feedback).map(|_| ()))
            .unwrap();
    }

    /// The session's activity time.
    pub fn activity(&self) -> i64 {
        self.app
            .store()
            .session(self.session_id)
            .unwrap()
            .session
            .last_activity_at
    }

    pub fn title(&self) -> String {
        self.app.store().item(self.item_id).unwrap().title
    }
}
