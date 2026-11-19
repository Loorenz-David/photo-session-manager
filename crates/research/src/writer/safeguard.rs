//! W2·U5: the safeguard (`master_plan_writer.md` §5 WC6; book 08C §6; HW3). [`filter`] decides,
//! before anything is stored, which findings and questions of a writer run survive: never one
//! about a key the photographer reviewed, never one already answered or dismissed, never more
//! asks than `max_questions`.

use std::collections::BTreeMap;

use psm_core::mechanisms::name_key::name_key;
use serde::{Deserialize, Serialize};

use crate::writer::{config::SafeguardModes, note::Decision};

/// The five finding types, in the order of 08C §6's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    FieldVsPhotos,
    FieldVsField,
    FieldVsListings,
    MissingInText,
    TextVsFacts,
}

impl FindingType {
    pub const ALL: [FindingType; 5] = [
        Self::FieldVsPhotos,
        Self::FieldVsField,
        Self::FieldVsListings,
        Self::MissingInText,
        Self::TextVsFacts,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FieldVsPhotos => "field_vs_photos",
            Self::FieldVsField => "field_vs_field",
            Self::FieldVsListings => "field_vs_listings",
            Self::MissingInText => "missing_in_text",
            Self::TextVsFacts => "text_vs_facts",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == text)
    }

    /// `field_vs_*`: about fields (the subject is `K`).
    pub fn about_fields(self) -> bool {
        matches!(
            self,
            Self::FieldVsPhotos | Self::FieldVsField | Self::FieldVsListings
        )
    }
}

/// What a finding type does when it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Off,
    Note,
    Ask,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Note => "note",
            Self::Ask => "ask",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Off, Self::Note, Self::Ask]
            .into_iter()
            .find(|m| m.as_str() == text)
    }

    /// The mode of a type under the settings (a value that is not a mode: the type's default).
    pub fn of(modes: &SafeguardModes, finding_type: FindingType) -> Self {
        modes
            .of(finding_type.as_str())
            .and_then(Self::parse)
            .or_else(|| {
                SafeguardModes::default()
                    .of(finding_type.as_str())
                    .and_then(Self::parse)
            })
            .unwrap_or(Self::Note)
    }
}

/// The life of a stored finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingState {
    Open,
    Answered,
    Dismissed,
    Superseded,
}

impl FindingState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Answered => "answered",
            Self::Dismissed => "dismissed",
            Self::Superseded => "superseded",
        }
    }
}

/// A finding (`Some(type)`) or a question (`None`) the run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub finding_type: Option<FindingType>,
    pub keys: Vec<String>,
    /// The text key a `missing_in_text` or `text_vs_facts` finding is about (`""` otherwise).
    pub text_field: String,
    pub text: String,
}

/// What the filter knows of one key, read in the run's commit transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyState {
    /// WC2 `reviewed` (HW3).
    pub reviewed: bool,
    pub decision: Decision,
}

/// Everything [`filter`] reads besides the candidates.
#[derive(Debug, Clone)]
pub struct FilterInput<'a> {
    pub modes: &'a SafeguardModes,
    /// The item's layout keys.
    pub layout: &'a [String],
    pub states: &'a BTreeMap<String, KeyState>,
    /// The identities of the item's `answered` and `dismissed` findings.
    pub resolved: &'a [String],
    pub max_questions: u32,
    /// The text keys this run wrote or found unchanged (write outcome `written` / `unchanged`).
    pub written: &'a [String],
}

/// A finding or question to store, state `open`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Survivor {
    /// `finding` or `question`.
    pub kind: String,
    /// A [`FindingType`] name, or `question`.
    #[serde(rename = "type")]
    pub finding_type: String,
    /// Sorted, deduplicated, layout keys only.
    pub keys: Vec<String>,
    pub text_field: String,
    pub identity: String,
    pub text: String,
    pub mode: Mode,
}

/// The filter's outcome: what to store, and the questions beyond the cap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filtered {
    pub stored: Vec<Survivor>,
    pub dropped_questions: Vec<String>,
}

/// WC6 identity: `type|<K sorted, comma-joined>|tf`; a question's `question|<K>||<N(text)>`.
pub fn identity(candidate: &Candidate, keys: &[String]) -> String {
    match candidate.finding_type {
        Some(t) => format!("{}|{}|{}", t.as_str(), keys.join(","), candidate.text_field),
        None => format!("question|{}||{}", keys.join(","), name_key(&candidate.text)),
    }
}

/// WC6 items 1–6: deterministic, before storing.
pub fn filter(candidates: &[Candidate], input: &FilterInput<'_>) -> Filtered {
    let state = |key: &str| input.states.get(key).copied();
    let mut seen: Vec<String> = Vec::new();
    let mut kept: Vec<(Option<FindingType>, Mode, Survivor)> = Vec::new();
    for candidate in candidates {
        let mut keys: Vec<String> = candidate
            .keys
            .iter()
            .filter(|k| input.layout.contains(k))
            .cloned()
            .collect();
        keys.sort();
        keys.dedup();
        // 1. A type that is off.
        let mode = match candidate.finding_type {
            Some(t) => Mode::of(input.modes, t),
            None => Mode::Ask,
        };
        if mode == Mode::Off {
            continue;
        }
        // 2. A subject key the photographer reviewed (HW3).
        let subject: Vec<String> = match candidate.finding_type {
            Some(t) if !t.about_fields() => {
                if candidate.text_field.is_empty() {
                    Vec::new()
                } else {
                    vec![candidate.text_field.clone()]
                }
            }
            _ => keys.clone(),
        };
        if subject.iter().any(|k| state(k).is_some_and(|s| s.reviewed)) {
            continue;
        }
        // 2 (amended 2026-11-19, W5·U4). A text finding about a text this run did not write
        // (e.g. the owner's own text) is not raised.
        if candidate.finding_type.is_some_and(|t| !t.about_fields())
            && !candidate.text_field.is_empty()
            && !input.written.contains(&candidate.text_field)
        {
            continue;
        }
        if let Some(t) = candidate.finding_type {
            // 4. A finding about fields names at least one.
            if t.about_fields() && keys.is_empty() {
                continue;
            }
            // 3. Photos are compared only with undecided values.
            if t == FindingType::FieldVsPhotos
                && !keys
                    .iter()
                    .all(|k| state(k).is_some_and(|s| s.decision.is_undecided()))
            {
                continue;
            }
        }
        // 5. Never again once answered or dismissed; once per run.
        let id = identity(candidate, &keys);
        if input.resolved.contains(&id) || seen.contains(&id) {
            continue;
        }
        seen.push(id.clone());
        kept.push((
            candidate.finding_type,
            mode,
            Survivor {
                kind: if candidate.finding_type.is_some() {
                    "finding"
                } else {
                    "question"
                }
                .to_owned(),
                finding_type: candidate
                    .finding_type
                    .map_or("question", FindingType::as_str)
                    .to_owned(),
                keys,
                text_field: candidate.text_field.clone(),
                identity: id,
                text: candidate.text.clone(),
                mode,
            },
        ));
    }
    // 6. The cap: asks in type order, then identity, then the questions in reply order.
    let mut asks: Vec<usize> = (0..kept.len())
        .filter(|i| kept[*i].0.is_some() && kept[*i].1 == Mode::Ask)
        .collect();
    asks.sort_by(|a, b| {
        (kept[*a].0, &kept[*a].2.identity).cmp(&(kept[*b].0, &kept[*b].2.identity))
    });
    asks.extend((0..kept.len()).filter(|i| kept[*i].0.is_none()));
    let limit = usize::try_from(input.max_questions).unwrap_or(usize::MAX);
    let mut dropped: Vec<usize> = Vec::new();
    for (n, i) in asks.into_iter().enumerate() {
        if n < limit {
            continue;
        }
        if kept[i].0.is_some() {
            kept[i].2.mode = Mode::Note;
        } else {
            dropped.push(i);
        }
    }
    let mut out = Filtered::default();
    for (i, (_, _, survivor)) in kept.into_iter().enumerate() {
        if dropped.contains(&i) {
            out.dropped_questions.push(survivor.text);
        } else {
            out.stored.push(survivor);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WOOD: &str = "custom.wood";
    const ERA: &str = "custom.era";
    const DESIGNER: &str = "custom.designer";
    const REPORT: &str = "custom.condition_report_description";

    fn layout() -> Vec<String> {
        [WOOD, ERA, DESIGNER, REPORT]
            .iter()
            .map(|k| (*k).to_owned())
            .collect()
    }

    fn finding(t: FindingType, keys: &[&str], text_field: &str) -> Candidate {
        Candidate {
            finding_type: Some(t),
            keys: keys.iter().map(|k| (*k).to_owned()).collect(),
            text_field: text_field.into(),
            text: format!("{} on {}", t.as_str(), keys.join(",")),
        }
    }

    fn question(keys: &[&str], text: &str) -> Candidate {
        Candidate {
            finding_type: None,
            keys: keys.iter().map(|k| (*k).to_owned()).collect(),
            text_field: String::new(),
            text: text.into(),
        }
    }

    fn state(reviewed: bool, decision: Decision) -> KeyState {
        KeyState { reviewed, decision }
    }

    fn run(
        candidates: &[Candidate],
        modes: &SafeguardModes,
        states: &[(&str, KeyState)],
        resolved: &[&str],
        max_questions: u32,
    ) -> Filtered {
        let states: BTreeMap<String, KeyState> =
            states.iter().map(|(k, s)| ((*k).to_owned(), *s)).collect();
        let resolved: Vec<String> = resolved.iter().map(|r| (*r).to_owned()).collect();
        filter(
            candidates,
            &FilterInput {
                modes,
                layout: &layout(),
                states: &states,
                resolved: &resolved,
                max_questions,
                written: &written(),
            },
        )
    }

    /// Every text key these tests' findings name, as written by the run.
    fn written() -> Vec<String> {
        [REPORT, "product.title", "product.description"]
            .iter()
            .map(|k| (*k).to_owned())
            .collect()
    }

    fn kept(filtered: &Filtered) -> Vec<(String, Mode)> {
        filtered
            .stored
            .iter()
            .map(|s| (s.finding_type.clone(), s.mode))
            .collect()
    }

    #[test]
    fn w2_c21a_a_type_that_is_off_is_dropped() {
        let modes = SafeguardModes {
            field_vs_field: "off".into(),
            ..SafeguardModes::default()
        };
        let out = run(
            &[finding(FindingType::FieldVsField, &[ERA, DESIGNER], "")],
            &modes,
            &[],
            &[],
            3,
        );
        assert_eq!(out, Filtered::default());
    }

    #[test]
    fn w5_rule2_a_text_finding_about_a_text_not_written_this_run_is_dropped() {
        let states: BTreeMap<String, KeyState> = BTreeMap::new();
        let candidates = [
            finding(FindingType::TextVsFacts, &[], "product.description"),
            finding(FindingType::TextVsFacts, &[], "product.title"),
        ];
        let written = vec!["product.title".to_owned()];
        let out = filter(
            &candidates,
            &FilterInput {
                modes: &SafeguardModes::default(),
                layout: &layout(),
                states: &states,
                resolved: &[],
                max_questions: 3,
                written: &written,
            },
        );
        assert_eq!(
            out.stored
                .iter()
                .map(|s| s.text_field.as_str())
                .collect::<Vec<_>>(),
            ["product.title"]
        );
    }

    #[test]
    fn w2_c21b_photos_are_never_set_against_a_decided_value() {
        let candidates = [finding(FindingType::FieldVsPhotos, &[WOOD], "")];
        let modes = SafeguardModes::default();
        let accepted = run(
            &candidates,
            &modes,
            &[(WOOD, state(true, Decision::Accepted))],
            &[],
            3,
        );
        assert_eq!(kept(&accepted), []);
        let typed = run(
            &candidates,
            &modes,
            &[(WOOD, state(false, Decision::Typed))],
            &[],
            3,
        );
        assert_eq!(kept(&typed), [("field_vs_photos".to_owned(), Mode::Ask)]);
        assert_eq!(typed.stored[0].identity, "field_vs_photos|custom.wood|");
    }

    #[test]
    fn w2_c21c_a_text_the_photographer_accepted_is_not_questioned() {
        let candidates = [finding(FindingType::MissingInText, &[ERA], REPORT)];
        let modes = SafeguardModes::default();
        let accepted = run(
            &candidates,
            &modes,
            &[(REPORT, state(true, Decision::Accepted))],
            &[],
            3,
        );
        assert_eq!(kept(&accepted), []);
        let suggested = run(
            &candidates,
            &modes,
            &[(REPORT, state(false, Decision::Unchecked(None)))],
            &[],
            3,
        );
        assert_eq!(
            kept(&suggested),
            [("missing_in_text".to_owned(), Mode::Ask)]
        );
    }

    #[test]
    fn w2_c21d_an_answered_or_dismissed_finding_is_never_raised_again() {
        // `resolved` holds the identities of the item's answered and dismissed rows alike (the
        // query reads both states); here one equal to the candidate's, then the candidate again
        // beside another resolved identity.
        let candidates = [finding(FindingType::FieldVsListings, &[DESIGNER], "")];
        let modes = SafeguardModes::default();
        let answered = run(
            &candidates,
            &modes,
            &[],
            &["field_vs_listings|custom.designer|"],
            3,
        );
        assert_eq!(kept(&answered), []);
        let dismissed = run(
            &candidates,
            &modes,
            &[],
            &[
                "field_vs_field|custom.era|",
                "field_vs_listings|custom.designer|",
            ],
            3,
        );
        assert_eq!(kept(&dismissed), []);
        assert_eq!(run(&candidates, &modes, &[], &[], 3).stored.len(), 1);
    }

    #[test]
    fn w2_c21e_at_most_max_questions_asks_and_questions() {
        let modes = SafeguardModes::default();
        let candidates = [
            finding(FindingType::TextVsFacts, &[], "product.title"),
            finding(FindingType::MissingInText, &[WOOD], REPORT),
            finding(FindingType::TextVsFacts, &[], "product.description"),
            finding(FindingType::MissingInText, &[ERA], REPORT),
            question(&[DESIGNER], "Who designed it?"),
            question(&[], "Is it a set?"),
        ];
        let out = run(&candidates, &modes, &[], &[], 3);
        let modes_of: Vec<(String, String, Mode)> = out
            .stored
            .iter()
            .map(|s| (s.finding_type.clone(), s.identity.clone(), s.mode))
            .collect();
        assert_eq!(
            modes_of,
            [
                (
                    "text_vs_facts".to_owned(),
                    "text_vs_facts||product.title".to_owned(),
                    Mode::Note
                ),
                (
                    "missing_in_text".to_owned(),
                    format!("missing_in_text|custom.wood|{REPORT}"),
                    Mode::Ask
                ),
                (
                    "text_vs_facts".to_owned(),
                    "text_vs_facts||product.description".to_owned(),
                    Mode::Ask
                ),
                (
                    "missing_in_text".to_owned(),
                    format!("missing_in_text|custom.era|{REPORT}"),
                    Mode::Ask
                ),
            ]
        );
        assert_eq!(out.dropped_questions, ["Who designed it?", "Is it a set?"]);
    }

    #[test]
    fn w2_c21f_a_finding_about_no_layout_field_is_dropped() {
        let out = run(
            &[finding(FindingType::FieldVsField, &["custom.gone"], "")],
            &SafeguardModes::default(),
            &[],
            &[],
            3,
        );
        assert_eq!(out, Filtered::default());
    }

    #[test]
    fn wc6_5_a_questions_identity_holds_its_words() {
        let a = question(&[], "Is it a set?");
        let b = question(&[], "Who made it?");
        assert_eq!(identity(&a, &[]), "question|||is it a set");
        assert_ne!(identity(&a, &[]), identity(&b, &[]));
    }
}
