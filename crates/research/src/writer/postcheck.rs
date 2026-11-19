//! W2·U2: the deterministic post-check (`master_plan_writer.md` §5 WC5 items 1–11; book 08C
//! §5.4, §7; evidence §7.4, §7.5). Pure: the same texts and context give the same hits.
//!
//! "Contains" is case-insensitive on NFC text; a word match uses Unicode word boundaries (a word
//! character is a letter, a digit or `_`); a phrase match is a substring match. Item 11
//! (Restoration) is W6 (WC8-R).

use std::collections::BTreeMap;

use psm_core::mechanisms::name_key::name_key;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::writer::{config::Lengths, defaults::restoration_phrases};

/// What a hit is about (`master_plan_writer.md` §4 "Enums").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitCode {
    Length,
    Banned,
    Dash,
    Punctuation,
    Pattern,
    UseSentences,
    Damage,
    Grade,
    Stability,
    Names,
    Years,
    Models,
    Restoration,
    NotRechecked,
}

/// One problem of one text. `key` names the Condition field of a `Damage` hit (it becomes a
/// `missing_in_text` finding on that field, WC5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hit {
    /// The text's output name (`title`, `description`, …).
    pub text: String,
    pub code: HitCode,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

/// A visible Condition-step severity choice field and its value (WC5.7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverityField {
    pub key: String,
    pub label: String,
    pub value: String,
    /// The value is a `fact` (WC2): only facts are checked.
    pub fact: bool,
}

/// What the texts are checked against.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CheckContext {
    pub language: String,
    pub lengths: Lengths,
    pub banned_extra: Vec<String>,
    /// The display values of the fields whose value is a fact.
    pub facts: Vec<String>,
    /// The note texts the writer received (after the owner's replacements).
    pub notes: Vec<String>,
    pub severity: Vec<SeverityField>,
    /// `{<key>: {<language>: [term]}}`.
    pub damage_terms: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// The detailed-condition field's value when that field is visible and holds a fact.
    pub detailed: Option<String>,
    /// The choices of the attribution definitions and the values of those keys on any item.
    pub attribution_names: Vec<String>,
    /// The selected "Restoration done" choices when that field is visible and holds a fact
    /// (WC5.11, WC8-R).
    #[serde(default)]
    pub restoration: Vec<String>,
    /// A23: rule 9 (stability) runs only when the writer's `stability_sentence` switch is on.
    /// (Rule 8 runs only when `detailed` is set, which the run does only with `grade_word` on.)
    #[serde(default)]
    pub stability: bool,
}

const BANNED_WORDS: [&str; 33] = [
    "timeless",
    "elevate",
    "elevates",
    "testament",
    "boasts",
    "stunning",
    "iconic",
    "nestled",
    "vibrant",
    "showcase",
    "showcases",
    "showcasing",
    "seamless",
    "effortless",
    "versatile",
    "must-have",
    "exquisite",
    "impeccable",
    "meticulous",
    "curated",
    "delve",
    "delves",
    "tapestry",
    "embodies",
    "exemplifies",
    "beloved",
    "gem",
    "treasure",
    "tidlös",
    "ikonisk",
    "fantastisk",
    "otrolig",
    "unik",
];
const BANNED_PHRASES: [&str; 12] = [
    "statement piece",
    "perfect for",
    "adds a touch of",
    "warmth and character",
    "piece of history",
    "whether you",
    "look no further",
    "en riktig pärla",
    "ett måste",
    "perfekt för",
    "lyfter rummet",
    "tillför värme",
];
/// Banned unless a fact value contains the word.
const EXEMPT_BY_FACT: [&str; 3] = ["iconic", "ikonisk", "unik"];
const PARTICIPLES: [&str; 14] = [
    "highlighting",
    "adding",
    "showcasing",
    "offering",
    "featuring",
    "creating",
    "bringing",
    "providing",
    "reflecting",
    "ensuring",
    "making",
    "giving",
    "lending",
    "boasting",
];
const USE_MARKERS_EN: [&str; 9] = [
    " suits ",
    " works at ",
    " works in ",
    " works as ",
    " use it ",
    " use them ",
    " ideal for ",
    " fits ",
    " can be used ",
];
const USE_MARKERS_SV: [&str; 3] = [" passar ", " fungerar ", " kan användas "];
const STABILITY_EN: [&str; 7] = [
    "stable", "sturdy", "firm", "sound", "wobble", "wobbles", "loose",
];
const STABILITY_SV: [&str; 3] = ["stadig", "stabil", "glapp"];
/// English damage words → Swedish terms (WC5.7; `|` separates alternatives).
const DAMAGE_SV: [(&str, &str); 17] = [
    ("scratch", "rep"),
    ("stain", "fläck"),
    ("discoloration", "missfärg"),
    ("veneer", "fanér"),
    ("dent", "buckl"),
    ("chip", "nagg"),
    ("crack", "spricka"),
    ("water", "vatten"),
    ("damage", "skad"),
    ("stability", "stadig|glapp"),
    ("missing", "saknas"),
    ("part", "del"),
    ("upholstery", "klädsel|stoppning"),
    ("odor", "lukt"),
    ("mold", "mögel"),
    ("function", "funktion|fungerar"),
    ("wear", "slit"),
];
/// The grade words of the detailed condition in Swedish (WC5.8).
const GRADES_SV: [(&str, &str); 4] = [
    ("excellent", "Utmärkt"),
    ("very good", "Mycket gott"),
    ("good", "Gott"),
    ("worn", "Slitet"),
];
/// Single letters that are dimensions, never model numbers (WC5.10).
/// WC5.10 (iii): single letters that are never a model token's letters (dimension letters, and
/// the English words "A" and "I", amended 2026-11-19).
const DIMENSION_LETTERS: [&str; 7] = ["W", "D", "H", "B", "L", "A", "I"];

/// WC5.10 (iii): whether a unit (`\s?(mm|cm|m|kg|%)\b`) follows position `at` of `c`.
fn unit_follows(c: &[char], at: usize) -> bool {
    let mut k = at;
    if k < c.len() && c[k].is_whitespace() {
        k += 1;
    }
    if c.get(k) == Some(&'%') {
        return true;
    }
    ["mm", "cm", "kg", "m"].iter().any(|unit| {
        let u: Vec<char> = unit.chars().collect();
        c.len() >= k + u.len()
            && c[k..k + u.len()] == u[..]
            && c.get(k + u.len()).is_none_or(|next| !is_word(*next))
    })
}

fn lower(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The byte offsets where `needle` occurs in `hay` with a word boundary at each end of `needle`
/// that is a word character.
fn positions(hay: &str, needle: &str) -> Vec<usize> {
    let Some(first) = needle.chars().next() else {
        return Vec::new();
    };
    let last = needle.chars().next_back().unwrap_or(first);
    hay.match_indices(needle)
        .filter(|(at, _)| {
            let before = hay[..*at].chars().next_back();
            let after = hay[at + needle.len()..].chars().next();
            (!is_word(first) || !before.is_some_and(is_word))
                && (!is_word(last) || !after.is_some_and(is_word))
        })
        .map(|(at, _)| at)
        .collect()
}

fn has_word(hay: &str, needle: &str) -> bool {
    !positions(hay, needle).is_empty()
}

/// Whitespace-separated tokens that contain a letter or a digit.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .count()
}

/// Sentences: split after ".", "!" or "?" followed by whitespace, or at the end.
fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if matches!(c, '.' | '!' | '?') && chars.peek().is_none_or(|(_, n)| n.is_whitespace()) {
            let end = at + c.len_utf8();
            out.push(&text[start..end]);
            start = end;
        }
    }
    out.push(&text[start..]);
    out.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// A sentence for marker search: lower case, every other character a space, padded.
fn marker_form(sentence: &str) -> String {
    let spaced: String = lower(sentence)
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    format!(
        " {} ",
        spaced.split_whitespace().collect::<Vec<_>>().join(" ")
    )
}

fn hit(text: &str, code: HitCode, detail: String) -> Hit {
    Hit {
        text: text.to_owned(),
        code,
        detail,
        key: None,
    }
}

fn swedish(language: &str) -> bool {
    language == "Swedish"
}

/// WC5.1.
fn length(name: &str, text: &str, lengths: &Lengths) -> Option<Hit> {
    let chars = text.chars().count();
    let words = word_count(text);
    let (count, min, max, unit) = match name {
        "title" => (chars, 0, lengths.title_max, "characters"),
        "description" => (
            words,
            lengths.description_min_words,
            lengths.description_max_words,
            "words",
        ),
        "condition_report" => (words, 0, lengths.condition_max_words, "words"),
        "seo_title" => (chars, 0, lengths.seo_title_max, "characters"),
        "seo_description" => (
            chars,
            lengths.seo_description_min,
            lengths.seo_description_max,
            "characters",
        ),
        _ => return None,
    };
    let count = u32::try_from(count).unwrap_or(u32::MAX);
    if count >= min && count <= max {
        return None;
    }
    let range = if min == 0 {
        format!("at most {max}")
    } else {
        format!("{min} to {max}")
    };
    Some(hit(
        name,
        HitCode::Length,
        format!("{count} {unit} ({range})"),
    ))
}

/// WC5.2.
fn banned(name: &str, text: &str, ctx: &CheckContext, out: &mut Vec<Hit>) {
    let text = lower(text);
    let facts: Vec<String> = ctx.facts.iter().map(|f| lower(f)).collect();
    let mut terms: Vec<String> = Vec::new();
    for word in BANNED_WORDS {
        if !has_word(&text, word) {
            continue;
        }
        if EXEMPT_BY_FACT.contains(&word) && facts.iter().any(|f| has_word(f, word)) {
            continue;
        }
        terms.push(word.to_owned());
    }
    for phrase in BANNED_PHRASES {
        if text.contains(phrase) {
            terms.push(phrase.to_owned());
        }
    }
    for extra in &ctx.banned_extra {
        let extra = lower(extra.trim());
        if !extra.is_empty() && has_word(&text, &extra) && !terms.contains(&extra) {
            terms.push(extra);
        }
    }
    for term in terms {
        out.push(hit(
            name,
            HitCode::Banned,
            format!("banned word \"{term}\""),
        ));
    }
}

/// WC5.3: an em dash anywhere; an en dash unless both neighbours are digits; " - ".
fn dash(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.iter().enumerate().any(|(i, c)| match c {
        '\u{2014}' => true,
        '\u{2013}' => {
            let digit = |j: Option<usize>| {
                j.and_then(|j| chars.get(j))
                    .is_some_and(char::is_ascii_digit)
            };
            !(digit(i.checked_sub(1)) && digit(Some(i + 1)))
        }
        _ => false,
    }) || text.contains(" - ")
}

/// WC5.5 (English): the pattern names found, one per pattern.
fn patterns(text: &str) -> Vec<&'static str> {
    let text = lower(text).replace('\u{2019}', "'");
    let mut found: Vec<&'static str> = Vec::new();
    for sentence in sentences(&text) {
        let after = |starts: Vec<(usize, usize)>, later: &[&str]| {
            starts.iter().any(|(at, len)| {
                later
                    .iter()
                    .any(|l| positions(sentence, l).iter().any(|q| *q >= at + len))
            })
        };
        let not_just: Vec<(usize, usize)> = positions(sentence, "not just")
            .into_iter()
            .map(|p| (p, "not just".len()))
            .collect();
        if after(not_just, &["but"]) && !found.contains(&"not just … but") {
            found.push("not just … but");
        }
        let mut negations: Vec<(usize, usize)> = Vec::new();
        for start in ["it's not", "it is not"] {
            negations.extend(
                positions(sentence, start)
                    .into_iter()
                    .map(|p| (p, start.len())),
            );
        }
        if after(negations, &["it's", "it is"]) && !found.contains(&"it's not … it's") {
            found.push("it's not … it's");
        }
        if has_word(sentence, "serves as") && !found.contains(&"serves as") {
            found.push("serves as");
        }
        for participle in PARTICIPLES {
            if has_word(sentence, &format!(", {participle}")) && !found.contains(&", <-ing>") {
                found.push(", <-ing>");
            }
        }
    }
    found
}

/// WC5.7: the terms that name a damage field in the report.
fn damage_terms(field: &SeverityField, ctx: &CheckContext) -> Vec<String> {
    if let Some(terms) = ctx
        .damage_terms
        .get(&field.key)
        .and_then(|by_language| by_language.get(&ctx.language))
    {
        return terms
            .iter()
            .map(|t| lower(t.trim()))
            .filter(|t| !t.is_empty())
            .collect();
    }
    let label = field.label.trim();
    let label = if label.to_lowercase().starts_with("condition.") {
        &label["condition.".len()..]
    } else {
        label
    };
    let english: Vec<String> = label
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| w.chars().count() >= 4)
        .map(|w| {
            let w = w.to_lowercase();
            if let Some(stem) = w.strip_suffix("es") {
                stem.to_owned()
            } else if let Some(stem) = w.strip_suffix('s') {
                stem.to_owned()
            } else {
                w
            }
        })
        .collect();
    if !swedish(&ctx.language) {
        return english;
    }
    english
        .iter()
        .filter_map(|w| DAMAGE_SV.iter().find(|(en, _)| en == w))
        .flat_map(|(_, sv)| sv.split('|').map(str::to_owned))
        .collect()
}

/// The grade word a report must open with (WC5.8).
fn grade_word(value: &str, language: &str) -> String {
    let grade = value.split(" - ").next().unwrap_or(value).trim();
    if swedish(language)
        && let Some((_, sv)) = GRADES_SV.iter().find(|(en, _)| *en == grade.to_lowercase())
    {
        return (*sv).to_owned();
    }
    grade.to_owned()
}

/// The maximal runs of ASCII digits of length 4 in `text`, with whether each stands alone
/// (no word character on either side).
fn four_digit_runs(text: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i - start == 4 {
                let alone = (start == 0 || !is_word(chars[start - 1]))
                    && (i == chars.len() || !is_word(chars[i]));
                out.push((chars[start..i].iter().collect(), alone));
            }
        } else {
            i += 1;
        }
    }
    out
}

/// The model-number tokens of one line (WC5.10 (iii)):
/// `\b[A-Z]{1,4}[- ]?\d{1,4}[A-Z]?\b`, minus a single W, D, H, B, L, A or I and minus a token
/// followed by a unit.
fn model_tokens(line: &str) -> Vec<String> {
    let c: Vec<char> = line.chars().collect();
    let n = c.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if !(c[i].is_ascii_uppercase() && (i == 0 || !is_word(c[i - 1]))) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && c[j].is_ascii_uppercase() {
            j += 1;
        }
        if j - i <= 4 {
            let mut k = j;
            if k + 1 < n && matches!(c[k], '-' | ' ') && c[k + 1].is_ascii_digit() {
                k += 1;
            }
            let mut d = k;
            while d < n && c[d].is_ascii_digit() {
                d += 1;
            }
            if (1..=4).contains(&(d - k)) {
                let mut end = d;
                if end < n && c[end].is_ascii_uppercase() && (end + 1 == n || !is_word(c[end + 1]))
                {
                    end += 1;
                }
                let letters: String = c[i..j].iter().collect();
                if (end == n || !is_word(c[end]))
                    && !DIMENSION_LETTERS.contains(&letters.as_str())
                    && !unit_follows(&c, end)
                {
                    out.push(c[i..end].iter().collect());
                }
            }
        }
        i = j;
    }
    out
}

fn squeezed(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, ' ' | '-'))
        .collect::<String>()
        .to_uppercase()
}

/// WC5.10: names, years and model numbers only from the facts and the note.
fn names_only_from_fields(name: &str, text: &str, ctx: &CheckContext, out: &mut Vec<Hit>) {
    let sources: Vec<&String> = ctx.facts.iter().chain(ctx.notes.iter()).collect();
    let keyed: Vec<String> = sources.iter().map(|s| name_key(s)).collect();
    let text_key = name_key(text);
    let mut seen: Vec<String> = Vec::new();
    for candidate in &ctx.attribution_names {
        let key = name_key(candidate);
        if key.is_empty() || seen.contains(&key) {
            continue;
        }
        seen.push(key.clone());
        if text_key.contains(&key) && !keyed.iter().any(|a| a.contains(&key)) {
            out.push(hit(
                name,
                HitCode::Names,
                format!("{} is not in the fields or the note", candidate.trim()),
            ));
        }
    }
    let known_years: Vec<String> = sources
        .iter()
        .flat_map(|s| four_digit_runs(s))
        .map(|(digits, _)| digits)
        .collect();
    let mut years: Vec<String> = Vec::new();
    for (digits, alone) in four_digit_runs(text) {
        if alone
            && ["18", "19", "20"].iter().any(|p| digits.starts_with(p))
            && !known_years.contains(&digits)
            && !years.contains(&digits)
        {
            years.push(digits);
        }
    }
    for year in years {
        out.push(hit(
            name,
            HitCode::Years,
            format!("{year} is not in the fields or the note"),
        ));
    }
    let known = squeezed(
        &sources
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let mut models: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Dimensions:") || trimmed.starts_with("Mått:") {
            continue;
        }
        for token in model_tokens(line) {
            if !known.contains(&squeezed(&token)) && !models.contains(&token) {
                models.push(token);
            }
        }
    }
    for token in models {
        out.push(hit(
            name,
            HitCode::Models,
            format!("{token} is not in the fields or the note"),
        ));
    }
}

/// WC5 items 1–11 over `texts` (`(output name, text)`), each trimmed.
pub fn check(texts: &[(String, String)], ctx: &CheckContext) -> Vec<Hit> {
    let mut out = Vec::new();
    for (name, text) in texts {
        let name = name.as_str();
        let text = text.trim();
        out.extend(length(name, text, &ctx.lengths));
        banned(name, text, ctx, &mut out);
        if dash(text) {
            out.push(hit(
                name,
                HitCode::Dash,
                "a dash used as punctuation".into(),
            ));
        }
        if text.contains('!') || text.contains('?') {
            out.push(hit(
                name,
                HitCode::Punctuation,
                "an exclamation mark or a question mark".into(),
            ));
        }
        if !swedish(&ctx.language) {
            for pattern in patterns(text) {
                out.push(hit(
                    name,
                    HitCode::Pattern,
                    format!("the pattern \"{pattern}\""),
                ));
            }
        }
        if name == "description" {
            let markers: &[&str] = if swedish(&ctx.language) {
                &USE_MARKERS_SV
            } else {
                &USE_MARKERS_EN
            };
            let count = sentences(text)
                .iter()
                .filter(|s| {
                    let form = marker_form(s);
                    markers.iter().any(|m| form.contains(m))
                })
                .count();
            if count > 1 {
                out.push(hit(
                    name,
                    HitCode::UseSentences,
                    format!("{count} sentences on where it can be used (at most one)"),
                ));
            }
        }
        if name == "condition_report" {
            let report = lower(text);
            for field in &ctx.severity {
                if !field.fact || matches!(field.value.as_str(), "None" | "N/A") {
                    continue;
                }
                let terms = damage_terms(field, ctx);
                if terms.is_empty() || terms.iter().any(|t| report.contains(t.as_str())) {
                    continue;
                }
                out.push(Hit {
                    key: Some(field.key.clone()),
                    ..hit(
                        name,
                        HitCode::Damage,
                        format!("{} is not in the condition report", field.label),
                    )
                });
            }
            if let Some(detailed) = &ctx.detailed {
                let grade = grade_word(detailed, &ctx.language);
                if !report.starts_with(&lower(&grade)) {
                    out.push(hit(
                        name,
                        HitCode::Grade,
                        format!("does not start with the grade \"{grade}\""),
                    ));
                }
            }
            let words: &[&str] = if swedish(&ctx.language) {
                &STABILITY_SV
            } else {
                &STABILITY_EN
            };
            if ctx.stability && !words.iter().any(|w| report.contains(w)) {
                out.push(hit(
                    name,
                    HitCode::Stability,
                    "says nothing about stability".into(),
                ));
            }
            for choice in &ctx.restoration {
                let phrases = restoration_phrases(choice, &ctx.language);
                let stated = if phrases.is_empty() {
                    report.contains(&lower(choice))
                } else {
                    phrases.iter().any(|p| report.contains(&lower(p)))
                };
                if !stated {
                    out.push(hit(
                        name,
                        HitCode::Restoration,
                        format!("the restoration step \"{choice}\" is not in the condition report"),
                    ));
                }
            }
        }
        names_only_from_fields(name, text, ctx, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::defaults::{EXAMPLES_EN, EXAMPLES_SV, ShippedExample};

    fn ctx() -> CheckContext {
        CheckContext {
            language: "English".into(),
            ..CheckContext::default()
        }
    }

    fn swedish_ctx() -> CheckContext {
        CheckContext {
            language: "Swedish".into(),
            ..CheckContext::default()
        }
    }

    fn one(name: &str, text: &str) -> Vec<(String, String)> {
        vec![(name.to_owned(), text.to_owned())]
    }

    fn codes(texts: &[(String, String)], ctx: &CheckContext) -> Vec<HitCode> {
        check(texts, ctx).into_iter().map(|h| h.code).collect()
    }

    /// `n` plain words ending in a full stop.
    fn filler(n: usize) -> String {
        format!("{}.", vec!["teak"; n].join(" "))
    }

    /// A description of 70 words that holds `sentence` (otherwise clean).
    fn description_with(sentence: &str) -> Vec<(String, String)> {
        let rest = 70 - word_count(sentence);
        one("description", &format!("{sentence} {}", filler(rest)))
    }

    fn report(text: &str) -> Vec<(String, String)> {
        one("condition_report", text)
    }

    fn severity(label: &str, value: &str, fact: bool) -> SeverityField {
        SeverityField {
            key: format!(
                "custom.{}",
                label.to_lowercase().replace([' ', '/', '.'], "_")
            ),
            label: label.into(),
            value: value.into(),
            fact,
        }
    }

    use HitCode::*;

    #[test]
    fn w2_c5a_title_length_in_characters() {
        let seventy = format!("{}ø", "a".repeat(69));
        assert_eq!(seventy.chars().count(), 70);
        assert_eq!(codes(&one("title", &seventy), &ctx()), []);
        assert_eq!(
            codes(&one("title", &format!("{seventy}a")), &ctx()),
            [Length]
        );
    }

    #[test]
    fn w2_c5b_description_length_in_words() {
        let at = |n: usize| codes(&one("description", &filler(n)), &ctx());
        assert_eq!(at(59), [Length]);
        assert_eq!(at(60), []);
        assert_eq!(at(120), []);
        assert_eq!(at(121), [Length]);
    }

    #[test]
    fn w2_c5c_condition_report_length_in_words() {
        let at = |n: usize| codes(&report(&format!("Good. {} Stable.", filler(n - 2))), &ctx());
        assert_eq!(at(45), []);
        assert_eq!(at(46), [Length]);
    }

    #[test]
    fn w2_c5d_search_title_length_in_characters() {
        assert_eq!(codes(&one("seo_title", &"a".repeat(60)), &ctx()), []);
        assert_eq!(codes(&one("seo_title", &"a".repeat(61)), &ctx()), [Length]);
    }

    #[test]
    fn w2_c5e_search_description_length_in_characters() {
        let at = |n: usize| codes(&one("seo_description", &"a".repeat(n)), &ctx());
        assert_eq!(at(119), [Length]);
        assert_eq!(at(120), []);
        assert_eq!(at(155), []);
        assert_eq!(at(156), [Length]);
    }

    #[test]
    fn w2_c6a_a_banned_word_in_any_case() {
        let hits = check(&description_with("A Stunning chair."), &ctx());
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Banned);
        assert!(
            hits[0].detail.contains("\"stunning\""),
            "{}",
            hits[0].detail
        );
    }

    #[test]
    fn w2_c6b_banned_phrases_in_both_languages_and_word_forms() {
        for sentence in [
            "It is perfect for a hall.",
            "Den är perfekt för en hall.",
            "It showcases the grain.",
        ] {
            assert_eq!(
                codes(&description_with(sentence), &ctx()),
                [Banned],
                "{sentence}"
            );
        }
    }

    #[test]
    fn w2_c6c_iconic_is_exempt_when_a_fact_says_it() {
        let title = one("title", "Iconic Chair in Teak");
        assert_eq!(codes(&title, &ctx()), [Banned]);
        let with_fact = CheckContext {
            facts: vec!["Iconic design".into()],
            ..ctx()
        };
        assert_eq!(codes(&title, &with_fact), []);
    }

    #[test]
    fn w2_c6d_the_owners_banned_words_are_checked_too() {
        let extra = CheckContext {
            banned_extra: vec!["nice".into()],
            ..ctx()
        };
        let hits = check(&one("title", "Nice Chair in Teak"), &extra);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Banned);
        assert!(hits[0].detail.contains("\"nice\""));
    }

    #[test]
    fn w2_c6e_dashes_as_punctuation() {
        let with_year = CheckContext {
            facts: vec!["1950s".into()],
            ..ctx()
        };
        let at = |title: &str| codes(&one("title", title), &with_year);
        assert_eq!(at("Chair in teak — oak"), [Dash]);
        assert_eq!(at("Chair, 1950–60"), []);
        assert_eq!(at("Chair in teak – oak"), [Dash]);
        assert_eq!(at("Chair in teak - oak"), [Dash]);
        assert_eq!(at("Chair, re-oiled"), []);
    }

    #[test]
    fn w2_c6f_exclamation_and_question_marks() {
        assert_eq!(codes(&one("title", "A fine chair!"), &ctx()), [Punctuation]);
        assert_eq!(codes(&one("title", "Ready?"), &ctx()), [Punctuation]);
    }

    #[test]
    fn w2_c6g_patterns() {
        for sentence in [
            "The top is oiled, highlighting the grain.",
            "It serves as a desk.",
            "This is not just a chair but a sculpture.",
            "It is not a copy, it is the original.",
        ] {
            assert_eq!(
                codes(&description_with(sentence), &ctx()),
                [Pattern],
                "{sentence}"
            );
        }
    }

    #[test]
    fn w2_c7a_one_use_sentence_is_allowed() {
        assert_eq!(codes(&description_with("It works at a desk."), &ctx()), []);
    }

    #[test]
    fn w2_c7b_two_use_sentences_are_one_hit() {
        assert_eq!(
            codes(
                &description_with("It suits a hallway. Use them as side tables."),
                &ctx()
            ),
            [UseSentences]
        );
    }

    #[test]
    fn w2_c7c_swedish_use_sentences() {
        assert_eq!(
            codes(
                &description_with("Den passar i hallen. Den passar vid ett bord."),
                &swedish_ctx()
            ),
            [UseSentences]
        );
    }

    #[test]
    fn w2_c8a_a_named_damage_is_covered() {
        let c = CheckContext {
            severity: vec![severity("Condition.Scratches", "Minor", true)],
            ..ctx()
        };
        assert_eq!(
            codes(&report("Good. Light scratches on the top. Stable."), &c),
            []
        );
    }

    #[test]
    fn w2_c8b_one_word_of_a_two_word_label_covers_it() {
        let c = CheckContext {
            severity: vec![severity("Condition.Veneer Damage", "Moderate", true)],
            ..ctx()
        };
        assert_eq!(
            codes(
                &report("Good. There is a 1 cm veneer chip on the top. Stable."),
                &c
            ),
            []
        );
    }

    #[test]
    fn w2_c8c_an_omitted_damage_is_a_hit_naming_the_field() {
        let c = CheckContext {
            severity: vec![severity("Condition.Stains/Discoloration", "Minor", true)],
            ..ctx()
        };
        let hits = check(&report("Good. Light wear on the top. Stable."), &c);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Damage);
        assert!(hits[0].detail.contains("Condition.Stains/Discoloration"));
        assert_eq!(
            hits[0].key.as_deref(),
            Some("custom.condition_stains_discoloration")
        );
    }

    #[test]
    fn w2_c8d_none_and_na_need_no_mention() {
        for value in ["None", "N/A"] {
            let c = CheckContext {
                severity: vec![severity("Condition.Scratches", value, true)],
                ..ctx()
            };
            assert_eq!(
                codes(&report("Good. Light wear. Stable."), &c),
                [],
                "{value}"
            );
        }
    }

    #[test]
    fn w2_c8e_swedish_terms() {
        let c = CheckContext {
            severity: vec![severity("Condition.Scratches", "Minor", true)],
            ..swedish_ctx()
        };
        assert_eq!(
            codes(&report("Gott skick. Lätta repor på sitsen. Stadig."), &c),
            []
        );
    }

    #[test]
    fn w2_c8f_the_owners_damage_terms() {
        let mut terms = BTreeMap::new();
        terms.insert(
            "custom.condition_odor".to_owned(),
            BTreeMap::from([("English".to_owned(), vec!["smell".to_owned()])]),
        );
        let c = CheckContext {
            severity: vec![severity("Condition.Odor", "Minor", true)],
            damage_terms: terms,
            ..ctx()
        };
        assert_eq!(
            codes(&report("Good. There is a faint smell. Stable."), &c),
            []
        );
    }

    #[test]
    fn w2_c8g_a_damage_that_is_not_a_fact_is_not_checked() {
        let c = CheckContext {
            severity: vec![severity("Condition.Cracks", "Major", false)],
            ..ctx()
        };
        assert_eq!(codes(&report("Good. Light wear. Stable."), &c), []);
    }

    const VERY_GOOD: &str =
        "Very Good - This vintage item has no defects, but it may show slight traces of use.";

    fn graded(language: &str) -> CheckContext {
        CheckContext {
            language: language.into(),
            detailed: Some(VERY_GOOD.into()),
            ..CheckContext::default()
        }
    }

    #[test]
    fn w2_c9a_the_report_opens_with_the_grade() {
        assert_eq!(
            codes(
                &report("Very good, restored. Light wear on the top. Stable."),
                &graded("English")
            ),
            []
        );
    }

    #[test]
    fn w2_c9b_another_grade_word_is_a_hit() {
        assert_eq!(
            codes(
                &report("Good. Light wear on the top. Stable."),
                &graded("English")
            ),
            [Grade]
        );
    }

    #[test]
    fn w2_c9c_the_swedish_grade_word() {
        assert_eq!(
            codes(
                &report("Mycket gott renoverat skick. Lätt slitage. Stadig."),
                &graded("Swedish")
            ),
            []
        );
    }

    #[test]
    fn w2_c9d_no_detailed_condition_no_grade_check() {
        assert_eq!(codes(&report("Light wear. Stable."), &ctx()), []);
    }

    #[test]
    fn w2_c9e_stability_is_stated_when_the_switch_is_on() {
        let on = CheckContext {
            stability: true,
            ..ctx()
        };
        assert_eq!(
            codes(&report("Good. Light wear on the top."), &on),
            [Stability]
        );
        assert_eq!(
            codes(&report("Good. Light wear. All three stand firm."), &on),
            []
        );
    }

    #[test]
    fn a23_no_stability_check_when_the_switch_is_off() {
        assert_eq!(
            codes(
                &report("Restored and refinished, ready to use. Light wear on the top."),
                &ctx()
            ),
            []
        );
    }

    fn attribution() -> CheckContext {
        CheckContext {
            attribution_names: vec!["Hans J. Wegner".into(), "Børge Mogensen".into()],
            facts: vec!["Børge Mogensen".into()],
            ..ctx()
        }
    }

    #[test]
    fn w2_c10a_a_designer_who_is_not_in_the_fields() {
        let hits = check(
            &one("title", "Hans J. Wegner Chair in Teak"),
            &attribution(),
        );
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Names);
        assert!(hits[0].detail.contains("Hans J. Wegner"));
    }

    #[test]
    fn w2_c10b_the_designer_in_the_fields() {
        assert_eq!(
            codes(
                &one("title", "Børge Mogensen Chair in Beech"),
                &attribution()
            ),
            []
        );
    }

    fn era() -> CheckContext {
        CheckContext {
            facts: vec!["1960s".into()],
            ..ctx()
        }
    }

    #[test]
    fn w2_c10c_a_year_that_is_not_in_the_fields() {
        let hits = check(&description_with("The chair was made in 1958."), &era());
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Years);
        assert!(hits[0].detail.contains("1958"));
    }

    #[test]
    fn w2_c10d_the_decade_of_the_fields() {
        assert_eq!(codes(&one("title", "Chair in Teak, 1960s"), &era()), []);
    }

    #[test]
    fn w2_c10e_model_numbers_only_from_the_fields() {
        let model = CheckContext {
            facts: vec!["J39".into()],
            ..ctx()
        };
        assert_eq!(codes(&one("title", "J39 Chair"), &model), []);
        let hits = check(&one("title", "GE 290 Chair"), &model);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].code, Models);
        assert!(hits[0].detail.contains("GE 290"));
    }

    #[test]
    fn w5_models_a_single_a_is_no_model_token() {
        assert_eq!(codes(&one("title", "A 1 chip."), &ctx()), []);
    }

    #[test]
    fn w5_models_a_single_i_is_no_model_token() {
        assert_eq!(codes(&one("title", "I 2 chairs."), &ctx()), []);
    }

    #[test]
    fn w5_models_a_token_followed_by_a_unit_is_no_model_token() {
        assert_eq!(codes(&one("title", "PK 22 cm"), &ctx()), []);
    }

    #[test]
    fn w2_c10f_the_dimensions_line_holds_no_model_number() {
        let text = format!(
            "{}\n\nDimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
            filler(60)
        );
        assert_eq!(codes(&one("description", &text), &ctx()), []);
    }

    #[test]
    fn w2_c10g_a_marking_the_note_names() {
        let c = CheckContext {
            facts: vec!["France & Søn".into()],
            notes: vec!["Stamp reads FD 164".into()],
            ..ctx()
        };
        assert_eq!(codes(&one("title", "France & Søn FD 164 Chair"), &c), []);
    }

    fn restored(language: &str, choices: &[&str]) -> CheckContext {
        CheckContext {
            language: language.into(),
            restoration: choices.iter().map(|c| (*c).to_owned()).collect(),
            ..CheckContext::default()
        }
    }

    fn report_and_description(report_text: &str, description: &str) -> Vec<(String, String)> {
        vec![
            ("description".to_owned(), description.to_owned()),
            ("condition_report".to_owned(), report_text.to_owned()),
        ]
    }

    #[test]
    fn w6_c5a_every_chosen_step_in_the_report_is_no_hit() {
        let texts = report_and_description(
            "Very Good condition. Frame sanded and oiled. Stable.",
            &filler(70),
        );
        let hits = check(&texts, &restored("English", &["Sanded", "Oiled"]));
        assert!(!hits.iter().any(|h| h.code == Restoration), "{hits:?}");
    }

    #[test]
    fn w6_c5b_a_step_only_the_description_states_is_one_hit_naming_it() {
        let texts = report_and_description(
            "Very Good condition. Frame sanded. Stable.",
            &format!("The frame was sanded and oiled. {}", filler(64)),
        );
        let hits: Vec<Hit> = check(&texts, &restored("English", &["Sanded", "Oiled"]))
            .into_iter()
            .filter(|h| h.code == Restoration)
            .collect();
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].text, "condition_report");
        assert!(hits[0].detail.contains("\"Oiled\""), "{}", hits[0].detail);
    }

    #[test]
    fn w6_c5c_a_swedish_phrase_states_the_step() {
        let hits = check(
            &report("Mycket gott renoverat skick. Sits och rygg nyklädda. Stadig."),
            &restored("Swedish", &["New fabric"]),
        );
        assert!(!hits.iter().any(|h| h.code == Restoration), "{hits:?}");
    }

    fn example_texts(e: &ShippedExample) -> Vec<(String, String)> {
        [
            ("title", e.title),
            ("description", e.description),
            ("condition_report", e.condition_report),
            ("seo_title", e.seo_title),
            ("seo_description", e.seo_description),
        ]
        .into_iter()
        .map(|(n, t)| (n.to_owned(), t.to_owned()))
        .collect()
    }

    #[test]
    fn w2_c11a_every_shipped_example_passes() {
        for (examples, language) in [(&EXAMPLES_EN[..], "English"), (&EXAMPLES_SV[..], "Swedish")] {
            for e in examples {
                let c = CheckContext {
                    language: language.into(),
                    facts: vec![e.fields.into()],
                    ..CheckContext::default()
                };
                assert_eq!(check(&example_texts(e), &c), [], "example {}", e.name);
            }
        }
    }

    #[test]
    fn w2_c11b_example_a_with_another_grade_word() {
        let a = &EXAMPLES_EN[0];
        let report_text = a.condition_report.replacen("Very", "Fair", 1);
        let c = CheckContext {
            language: "English".into(),
            facts: vec![a.fields.into()],
            detailed: Some(VERY_GOOD.into()),
            ..CheckContext::default()
        };
        assert_eq!(codes(&report(&report_text), &c), [Grade]);
    }
}
