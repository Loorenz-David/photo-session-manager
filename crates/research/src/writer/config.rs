//! W2·U1: the writer's settings (`master_plan_writer.md` §4 "Settings"). The `writer` slot of the
//! settings file holds a [`WriterSettings`]; it is read tolerantly by [`WriterSettings::from_slot`]
//! and checked by [`WriterSettings::validate`]. The app wires the slot in W3; until then the
//! research harness supplies it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::writer::defaults;

/// The languages the writer writes in (08C §8).
pub const LANGUAGES: [&str; 2] = ["English", "Swedish"];
/// The safeguard's finding types, in the order of 08C §6's table.
pub const FINDING_TYPES: [&str; 5] = [
    "field_vs_photos",
    "field_vs_field",
    "field_vs_listings",
    "missing_in_text",
    "text_vs_facts",
];
/// The modes a finding type can be set to.
pub const MODES: [&str; 3] = ["off", "note", "ask"];
/// The most questions a write may ask.
pub const MAX_QUESTIONS_LIMIT: u32 = 10;
/// The most earlier instruction texts kept per language (`master_plan_writer.md` §4).
pub const VERSIONS_MAX: usize = 20;

/// An earlier saved instruction text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionVersion {
    pub saved_at: i64,
    pub text: String,
}

/// The instruction of one language: `text: None` is the shipped default (§9).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Instruction {
    pub text: Option<String>,
    /// Newest first, at most 20.
    pub versions: Vec<InstructionVersion>,
}

/// One style example shown to the model (never copied).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Example {
    pub name: String,
    pub fields: String,
    pub title: String,
    pub description: String,
    pub condition_report: String,
    pub seo_title: String,
    pub seo_description: String,
}

/// Target lengths (WC5.1): characters for titles, words for the description and the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lengths {
    pub title_max: u32,
    pub description_min_words: u32,
    pub description_max_words: u32,
    pub condition_max_words: u32,
    pub seo_title_max: u32,
    pub seo_description_min: u32,
    pub seo_description_max: u32,
}

impl Default for Lengths {
    fn default() -> Self {
        Self {
            title_max: 70,
            description_min_words: 60,
            description_max_words: 120,
            condition_max_words: 45,
            seo_title_max: 60,
            seo_description_min: 120,
            seo_description_max: 155,
        }
    }
}

/// One mode (`off` | `note` | `ask`) per safeguard type (08C §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SafeguardModes {
    pub field_vs_photos: String,
    pub field_vs_field: String,
    pub field_vs_listings: String,
    pub missing_in_text: String,
    pub text_vs_facts: String,
}

impl Default for SafeguardModes {
    fn default() -> Self {
        Self {
            field_vs_photos: "ask".to_owned(),
            field_vs_field: "note".to_owned(),
            field_vs_listings: "note".to_owned(),
            missing_in_text: "ask".to_owned(),
            text_vs_facts: "ask".to_owned(),
        }
    }
}

impl SafeguardModes {
    /// The mode of a finding type by its stored name (`None` for an unknown type).
    pub fn of(&self, finding_type: &str) -> Option<&str> {
        Some(match finding_type {
            "field_vs_photos" => &self.field_vs_photos,
            "field_vs_field" => &self.field_vs_field,
            "field_vs_listings" => &self.field_vs_listings,
            "missing_in_text" => &self.missing_in_text,
            "text_vs_facts" => &self.text_vs_facts,
            _ => return None,
        })
    }
}

/// The `writer` slot (`master_plan_writer.md` §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WriterSettings {
    pub auto: bool,
    pub language: String,
    pub model: String,
    pub effort: String,
    pub instructions: BTreeMap<String, Instruction>,
    /// `None` for a language: the shipped examples (§9.4).
    pub examples: BTreeMap<String, Option<Vec<Example>>>,
    pub banned_extra: Vec<String>,
    pub lengths: Lengths,
    /// `None`: the default rule of WC2.
    pub visible_keys: Option<Vec<String>>,
    pub safeguard: SafeguardModes,
    pub max_questions: u32,
    pub shop_name: String,
    pub condition_report_key: String,
    /// `{<key>: {<language>: [term]}}` (WC5.7).
    pub damage_terms: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// A23 (owner 2026-11-19): the condition report starts with the Detailed condition's grade
    /// word, and post-check rule 8 checks it. Off: the grade word is never written.
    pub grade_word: bool,
    /// A23: the condition report ends with a stability sentence, and rule 9 checks it.
    pub stability_sentence: bool,
    /// A23: the built-in title formula is sent with the instruction.
    pub title_formula: bool,
    /// A23: the description ends with the dimensions line, whose format is sent.
    pub dimensions_line: bool,
}

/// A23: the `kind` of an exported writer settings file.
pub const EXPORT_KIND: &str = "session-manager-writer-settings";
/// A23: the format version of an exported writer settings file.
pub const EXPORT_VERSION: u32 = 1;

/// A23: the four switches, by their stored names.
pub const SWITCHES: [&str; 4] = [
    "grade_word",
    "stability_sentence",
    "title_formula",
    "dimensions_line",
];

impl Default for WriterSettings {
    fn default() -> Self {
        Self {
            auto: true,
            language: "English".to_owned(),
            model: "gpt-6.1-sol".to_owned(),
            effort: "medium".to_owned(),
            instructions: BTreeMap::new(),
            examples: BTreeMap::new(),
            banned_extra: Vec::new(),
            lengths: Lengths::default(),
            visible_keys: None,
            safeguard: SafeguardModes::default(),
            max_questions: 3,
            shop_name: String::new(),
            condition_report_key: "custom.condition_report_description".to_owned(),
            damage_terms: BTreeMap::new(),
            grade_word: false,
            stability_sentence: false,
            title_formula: false,
            dimensions_line: false,
        }
    }
}

/// What a writer run executes from (WC7): the settings as they were at its start, the
/// instruction text and the examples actually sent, and the origin that queued it. Part of the
/// run's settings snapshot (`ResearchConfig::writer`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WriterSnapshot {
    /// `next_to_review`, `all_checked`, `write_texts` or `answer`.
    pub origin: String,
    pub settings: WriterSettings,
    pub instruction: String,
    pub examples: Vec<Example>,
}

impl WriterSnapshot {
    /// What a writer request is built from under `settings`: the instruction and the examples of
    /// its language. The run's start and the preview both resolve it here.
    pub fn resolve(settings: &WriterSettings, origin: &str) -> Self {
        Self {
            origin: origin.to_owned(),
            instruction: settings.resolved_instruction(&settings.language),
            examples: settings.resolved_examples(&settings.language),
            settings: settings.clone(),
        }
    }
}

impl WriterSettings {
    /// The versions rule (08C §8): when `next` differs from `replaced`, `replaced` becomes the
    /// newest earlier version of `language` with time `saved_at`; at most [`VERSIONS_MAX`] are
    /// kept, the oldest dropped. Returns whether a version was kept.
    pub fn push_version(
        &mut self,
        language: &str,
        replaced: &str,
        next: &str,
        saved_at: i64,
    ) -> bool {
        if replaced == next {
            return false;
        }
        let versions = &mut self
            .instructions
            .entry(language.to_owned())
            .or_default()
            .versions;
        versions.insert(
            0,
            InstructionVersion {
                saved_at,
                text: replaced.to_owned(),
            },
        );
        versions.truncate(VERSIONS_MAX);
        true
    }

    /// The slot to store when `incoming` is saved over `self` (the stored slot) at `now`: the
    /// earlier versions are the stored ones (never the caller's); for each language whose
    /// instruction text changes, the stored text it replaces is kept as a version; a text equal
    /// to the shipped one is stored as the shipped default (`None`), and so is an example list
    /// equal to the shipped list.
    pub fn saved_over(&self, mut incoming: WriterSettings, now: i64) -> WriterSettings {
        let mut languages: Vec<String> = LANGUAGES.iter().map(|l| (*l).to_owned()).collect();
        for language in self.instructions.keys().chain(incoming.instructions.keys()) {
            if !languages.contains(language) {
                languages.push(language.clone());
            }
        }
        for language in &languages {
            let replaced = self.resolved_instruction(language);
            let next = incoming.resolved_instruction(language);
            let shipped = defaults::instruction(language);
            let text = (next != shipped).then(|| next.clone());
            let versions = self
                .instructions
                .get(language)
                .map(|i| i.versions.clone())
                .unwrap_or_default();
            incoming
                .instructions
                .insert(language.clone(), Instruction { text, versions });
            incoming.push_version(language, &replaced, &next, now);
        }
        for (language, list) in incoming.examples.iter_mut() {
            if list.as_ref() == Some(&defaults::examples(language)) {
                *list = None;
            }
        }
        incoming
    }

    /// **Reset to default** (08C §8): the saved text of `language`, if any, is kept as the newest
    /// earlier version and the instruction returns to the shipped one.
    pub fn reset_instruction(&mut self, language: &str, now: i64) {
        let saved = self.instructions.get(language).and_then(|i| i.text.clone());
        if let Some(saved) = saved {
            self.push_version(language, &saved, defaults::instruction(language), now);
        }
        self.instructions
            .entry(language.to_owned())
            .or_default()
            .text = None;
    }

    /// Reads the slot tolerantly (as `ResearchSettings::from_slot`): a missing slot or field takes
    /// its default, and a field held in another shape is ignored with a warning.
    pub fn from_slot(slot: Option<&Json>) -> Self {
        let Some(Json::Object(given)) = slot else {
            return Self::default();
        };
        if let Ok(settings) = serde_json::from_value::<Self>(Json::Object(given.clone())) {
            return settings;
        }
        let mut merged = serde_json::to_value(Self::default()).unwrap_or_default();
        for (key, value) in given {
            let mut trial = merged.clone();
            trial[key] = value.clone();
            if serde_json::from_value::<Self>(trial.clone()).is_ok() {
                merged = trial;
            } else {
                tracing::warn!(key = %key, "writer setting has an unexpected shape; default used");
            }
        }
        serde_json::from_value(merged).unwrap_or_default()
    }

    /// Refuses a writer settings value (whole or a patch of top-level keys) whose language is not
    /// English or Swedish, whose safeguard mode is not off, note or ask, whose `max_questions` is
    /// outside 0–10, whose lengths hold a 0 or a minimum above its maximum, or whose model is blank.
    /// The text names the field.
    pub fn validate(value: &Json) -> Result<(), String> {
        let Some(object) = value.as_object() else {
            return Err("writer settings must be an object".to_owned());
        };
        if let Some(language) = object.get("language")
            && !language.as_str().is_some_and(|l| LANGUAGES.contains(&l))
        {
            return Err("language must be English or Swedish".to_owned());
        }
        if let Some(model) = object.get("model")
            && !model.as_str().is_some_and(|m| !m.trim().is_empty())
        {
            return Err("model must not be blank".to_owned());
        }
        if let Some(effort) = object.get("effort")
            && !effort.is_string()
        {
            return Err("effort must be a text".to_owned());
        }
        if let Some(max) = object.get("max_questions")
            && !max
                .as_u64()
                .is_some_and(|n| n <= u64::from(MAX_QUESTIONS_LIMIT))
        {
            return Err(format!(
                "max_questions must be a whole number from 0 to {MAX_QUESTIONS_LIMIT}"
            ));
        }
        if let Some(safeguard) = object.get("safeguard") {
            let Some(modes) = safeguard.as_object() else {
                return Err("safeguard must be an object".to_owned());
            };
            for (name, mode) in modes {
                if !FINDING_TYPES.contains(&name.as_str()) {
                    return Err(format!("safeguard.{name} is not a finding type"));
                }
                if !mode.as_str().is_some_and(|m| MODES.contains(&m)) {
                    return Err(format!("safeguard.{name} must be off, note or ask"));
                }
            }
        }
        if let Some(lengths) = object.get("lengths") {
            let Some(given) = lengths.as_object() else {
                return Err("lengths must be an object".to_owned());
            };
            let mut merged = serde_json::to_value(Lengths::default()).unwrap_or_default();
            for (name, length) in given {
                if merged.get(name).is_none() {
                    return Err(format!("lengths.{name} is not a length"));
                }
                if !length
                    .as_u64()
                    .is_some_and(|n| n >= 1 && n <= u64::from(u32::MAX))
                {
                    return Err(format!("lengths.{name} must be a whole number above 0"));
                }
                merged[name] = length.clone();
            }
            let number = |name: &str| merged[name].as_u64().unwrap_or(0);
            for (min, max) in [
                ("description_min_words", "description_max_words"),
                ("seo_description_min", "seo_description_max"),
            ] {
                if number(min) > number(max) {
                    let field = if given.contains_key(min) { min } else { max };
                    return Err(format!(
                        "lengths.{field}: lengths.{min} must not be above lengths.{max}"
                    ));
                }
            }
        }
        if let Some(visible) = object.get("visible_keys")
            && !visible.is_null()
            && !visible
                .as_array()
                .is_some_and(|keys| keys.iter().all(Json::is_string))
        {
            return Err("visible_keys must be a list of field keys".to_owned());
        }
        if let Some(key) = object.get("condition_report_key")
            && !key.as_str().is_some_and(|k| !k.trim().is_empty())
        {
            return Err("condition_report_key must not be blank".to_owned());
        }
        for switch in SWITCHES {
            if let Some(value) = object.get(switch)
                && !value.is_boolean()
            {
                return Err(format!("{switch} must be true or false"));
            }
        }
        Ok(())
    }

    /// A23 **Export…**: the file's content, `{kind, version, writer}`. `writer` is this slot
    /// without the earlier instruction versions (they stay on the Mac that made them). Nothing
    /// from another settings slot is ever part of it.
    pub fn export_value(&self) -> Json {
        let mut slot = self.clone();
        for instruction in slot.instructions.values_mut() {
            instruction.versions.clear();
        }
        serde_json::json!({
            "kind": EXPORT_KIND,
            "version": EXPORT_VERSION,
            "writer": slot,
        })
    }

    /// A23 **Import…**: the writer settings of an exported file, refused (naming the reason) when
    /// the file is not one, or when its `writer` object fails [`Self::validate`] or does not read
    /// as writer settings. Earlier versions in the file are ignored; the caller saves the result
    /// over the stored slot with [`Self::saved_over`], which keeps the stored versions.
    pub fn import_value(file: &Json) -> Result<WriterSettings, String> {
        if file.get("kind").and_then(Json::as_str) != Some(EXPORT_KIND) {
            return Err("this is not a Session Manager writer settings file".to_owned());
        }
        let Some(writer) = file.get("writer").filter(|w| w.is_object()) else {
            return Err("the file holds no writer settings".to_owned());
        };
        Self::validate(writer)?;
        let mut settings: WriterSettings = serde_json::from_value(writer.clone())
            .map_err(|e| format!("the writer settings in the file cannot be read: {e}"))?;
        for instruction in settings.instructions.values_mut() {
            instruction.versions.clear();
        }
        Ok(settings)
    }

    /// The instruction sent for `language`: the saved text, or the shipped one (§9).
    pub fn resolved_instruction(&self, language: &str) -> String {
        self.instructions
            .get(language)
            .and_then(|i| i.text.clone())
            .unwrap_or_else(|| defaults::instruction(language).to_owned())
    }

    /// The examples sent for `language`: the saved list, or the shipped one (§9.4).
    pub fn resolved_examples(&self, language: &str) -> Vec<Example> {
        self.examples
            .get(language)
            .cloned()
            .flatten()
            .unwrap_or_else(|| defaults::examples(language))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn w2_c1a_the_defaults_are_the_master_plans_list() {
        let settings = WriterSettings::default();
        assert_eq!(
            serde_json::to_value(&settings).unwrap(),
            json!({
                "auto": true,
                "language": "English",
                "model": "gpt-6.1-sol",
                "effort": "medium",
                "instructions": {},
                "examples": {},
                "banned_extra": [],
                "lengths": {
                    "title_max": 70,
                    "description_min_words": 60,
                    "description_max_words": 120,
                    "condition_max_words": 45,
                    "seo_title_max": 60,
                    "seo_description_min": 120,
                    "seo_description_max": 155,
                },
                "visible_keys": null,
                "safeguard": {
                    "field_vs_photos": "ask",
                    "field_vs_field": "note",
                    "field_vs_listings": "note",
                    "missing_in_text": "ask",
                    "text_vs_facts": "ask",
                },
                "max_questions": 3,
                "shop_name": "",
                "condition_report_key": "custom.condition_report_description",
                "damage_terms": {},
                "grade_word": false,
                "stability_sentence": false,
                "title_formula": false,
                "dimensions_line": false,
            })
        );
    }

    #[test]
    fn w2_c1b_the_shipped_instruction_and_examples_resolve_until_a_text_is_saved() {
        let mut settings = WriterSettings::default();
        assert_eq!(
            settings.resolved_instruction("English"),
            defaults::INSTRUCTION_EN
        );
        let names = |examples: Vec<Example>| -> Vec<String> {
            examples.into_iter().map(|e| e.name).collect()
        };
        assert_eq!(
            names(settings.resolved_examples("English")),
            ["A", "B", "C"]
        );
        assert_eq!(
            settings.resolved_instruction("Swedish"),
            defaults::INSTRUCTION_SV
        );
        assert_eq!(names(settings.resolved_examples("Swedish")), ["D"]);
        settings.instructions.insert(
            "English".into(),
            Instruction {
                text: Some("X".into()),
                versions: Vec::new(),
            },
        );
        assert_eq!(settings.resolved_instruction("English"), "X");
    }

    #[test]
    fn w2_c2a_a_missing_slot_is_the_default_and_a_wrongly_shaped_field_is_ignored() {
        assert_eq!(WriterSettings::from_slot(None), WriterSettings::default());
        let read = WriterSettings::from_slot(Some(
            &json!({"max_questions": "three", "shop_name": "Beyo"}),
        ));
        assert_eq!(
            read,
            WriterSettings {
                shop_name: "Beyo".into(),
                ..WriterSettings::default()
            }
        );
    }

    #[test]
    fn w4_c2b_push_version_keeps_twenty_newest_first_dropping_the_oldest() {
        let mut settings = WriterSettings::default();
        let stored: Vec<InstructionVersion> = (1..=20)
            .map(|n| InstructionVersion {
                saved_at: 1_000 - n,
                text: format!("V{n}"),
            })
            .collect();
        settings.instructions.insert(
            "English".into(),
            Instruction {
                text: Some("Current".into()),
                versions: stored.clone(),
            },
        );
        assert!(settings.push_version("English", "Current", "New", 5_000));
        let versions = &settings.instructions["English"].versions;
        assert_eq!(versions.len(), 20);
        assert_eq!(
            versions[0],
            InstructionVersion {
                saved_at: 5_000,
                text: "Current".into()
            }
        );
        assert_eq!(versions[1..], stored[..19]);
        assert!(!versions.iter().any(|v| v.text == "V20"));
    }

    #[test]
    fn w4_c2b_push_version_keeps_nothing_when_the_text_is_unchanged() {
        let mut settings = WriterSettings::default();
        assert!(!settings.push_version("English", "Same", "Same", 5_000));
        assert!(
            settings
                .instructions
                .get("English")
                .is_none_or(|i| i.versions.is_empty())
        );
    }

    #[test]
    fn w2_c2b_validate_refuses_each_bad_value_naming_its_field() {
        let cases = [
            (json!({"language": "German"}), "language"),
            (
                json!({"safeguard": {"field_vs_field": "loud"}}),
                "safeguard.field_vs_field",
            ),
            (json!({"max_questions": 11}), "max_questions"),
            (
                json!({"lengths": {"description_min_words": 130, "description_max_words": 120}}),
                "lengths.description_min_words",
            ),
            (json!({"model": ""}), "model"),
        ];
        for (value, field) in cases {
            let error = WriterSettings::validate(&value).expect_err(field);
            assert!(error.starts_with(field), "{field}: {error}");
        }
        assert_eq!(
            WriterSettings::validate(&serde_json::to_value(WriterSettings::default()).unwrap()),
            Ok(())
        );
    }

    #[test]
    fn a23_export_then_import_gives_the_same_settings_without_versions() {
        let mut settings = WriterSettings {
            shop_name: "Shop".into(),
            title_formula: true,
            banned_extra: vec!["sanded back".into()],
            ..WriterSettings::default()
        };
        settings.instructions.insert(
            "English".into(),
            Instruction {
                text: Some("Write plainly.".into()),
                versions: vec![InstructionVersion {
                    saved_at: 1,
                    text: "Old".into(),
                }],
            },
        );
        let file = settings.export_value();
        assert_eq!(file["kind"], json!(EXPORT_KIND));
        assert_eq!(
            file["writer"]["instructions"]["English"]["versions"],
            json!([])
        );
        let imported = WriterSettings::import_value(&file).unwrap();
        let mut expected = settings.clone();
        expected
            .instructions
            .get_mut("English")
            .unwrap()
            .versions
            .clear();
        assert_eq!(imported, expected);
    }

    #[test]
    fn a23_import_refuses_a_file_that_is_not_writer_settings() {
        for (file, start) in [
            (json!({"writer": {}}), "this is not"),
            (json!({"kind": EXPORT_KIND}), "the file holds no"),
            (
                json!({"kind": EXPORT_KIND, "writer": {"language": "German"}}),
                "language",
            ),
            (
                json!({"kind": EXPORT_KIND, "writer": {"grade_word": "no"}}),
                "grade_word",
            ),
            (
                json!({"kind": EXPORT_KIND, "writer": {"instructions": 3}}),
                "the writer settings in the file cannot be read",
            ),
        ] {
            let error = WriterSettings::import_value(&file).expect_err(start);
            assert!(error.starts_with(start), "{start}: {error}");
        }
    }

    #[test]
    fn a23_switches_default_off_and_must_be_booleans() {
        let settings = WriterSettings::from_slot(Some(&json!({"language": "English"})));
        assert!(!settings.grade_word);
        assert!(!settings.stability_sentence);
        assert!(!settings.title_formula);
        assert!(!settings.dimensions_line);
        for switch in SWITCHES {
            let error = WriterSettings::validate(&json!({ switch: "yes" })).expect_err(switch);
            assert!(error.starts_with(switch), "{switch}: {error}");
            assert_eq!(WriterSettings::validate(&json!({ switch: true })), Ok(()));
        }
    }
}
