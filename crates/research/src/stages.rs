//! The stages of one run (08A §5, MC-A7). Each stage is made of units; a unit's provider calls go
//! through the gateway, and when its last call returns one transaction commits its output, its
//! steps and the run's stage reached (MC-A7 commit rule). A committed unit never runs again, so
//! `execute` resumes at the first uncommitted unit.

use psm_core::{
    Result,
    domain::{
        attribute::{AttributeGroup, Definition},
        photo::Classification,
    },
    ports::store::StoreTx,
};
use serde_json::{Map, Value as Json, json};

use crate::{
    config::ResearchConfig,
    domain::{FieldClass, RunState, Stage, StepKind},
    gateway::{Answer, CallRequest, GatewayOutcome, PHOTO_REMOVED},
    images::{ResearchImage, eligible, source},
    ports::ModelRequest,
    rules::{
        confidence::field_class,
        fingerprint::fingerprint,
        sites::{Candidate, page_identity, reading_order, site_of},
        value_map::{Mapped, Stated, map_output, same_statement},
        write_table::never_written,
    },
    service::{FormSnapshot, ResearchService},
    store::{self, NewStep, OutputRow},
};

/// How `execute` ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteOutcome {
    Finished,
    StoppedAtCap,
    Failed,
    /// A call could not leave (offline): the run stays `running`; the job tries again.
    HoldOffline,
    /// The run is not `running` (nothing was done).
    NotRunning,
}

/// The instructions each stage sends (ours; versioned by `INSTRUCTIONS_VERSION`).
const OBSERVE: &str = "Describe what is visible in the photo for each listed field. Answer with \
    the field's own choice string when one names the same thing. Mark each read sure or unsure, \
    and say whether the photo shows a label, stamp or marking.";
const READ: &str = "For each listed field, quote what this listing page states and give the value \
    in the field's terms, or null when it names nothing the field allows.";
const COMPARE: &str = "Compare the piece in the photo with the listing's images: match, similar \
    or different, with the features compared.";
const LABELS: &str = "Transcribe the label, stamp or marking and give, per listed field, what it \
    states.";
const CONDITION: &str = "For each listed condition field, read the wear the condition photos show. \
    Mark each read sure or unsure.";
pub(crate) const DECIDE: &str = "Write one short paragraph per decided field explaining the \
    decision from the evidence given.";

/// A committed unit's output and fingerprint.
#[derive(Debug, Clone)]
pub(crate) struct Committed {
    pub stage: Stage,
    pub output: Json,
    pub fingerprint: String,
}

/// What a unit's work produced.
pub(crate) enum Work {
    Commit {
        output: Json,
        steps: Vec<NewStep>,
    },
    /// Stop rule: skip the remaining reading units.
    Skip,
    Halt(ExecuteOutcome),
}

pub(crate) enum Flow {
    Done(Committed),
    Skip,
    Halt(ExecuteOutcome),
}

/// The answer to one provider call, as a unit sees it.
pub(crate) enum Asked {
    Answer(Answer),
    NotDetermined(String),
    Work(Work),
}

/// One run being executed.
pub(crate) struct Ctx<'a> {
    pub svc: &'a ResearchService,
    pub run: store::RunRow,
    pub config: ResearchConfig,
    pub form: FormSnapshot,
}

pub(crate) fn step(stage: Stage, kind: StepKind, summary: impl Into<String>) -> NewStep {
    NewStep {
        stage: stage.as_str().to_owned(),
        kind,
        summary: summary.into(),
        detail: String::new(),
        source_url: None,
        source_title: None,
        photo_ids: Vec::new(),
        field_keys: Vec::new(),
    }
}

/// A field as a request describes it.
fn descriptor(definition: &Definition) -> Json {
    json!({
        "key": definition.key,
        "label": definition.label,
        "core_type": definition.core_type,
        "choices": definition.choices,
        "description": definition.description,
    })
}

/// The layout fields of one class, in layout order.
pub(crate) fn fields_of(ctx: &Ctx<'_>, classes: &[FieldClass]) -> Vec<Definition> {
    ctx.form
        .fields
        .iter()
        .filter(|f| classes.contains(&class_of(ctx, &f.definition, f.step)))
        .map(|f| f.definition.clone())
        .collect()
}

pub(crate) fn class_of(ctx: &Ctx<'_>, definition: &Definition, step: AttributeGroup) -> FieldClass {
    field_class(
        definition,
        never_written(definition, Some(step)),
        &ctx.config.attribution_keys,
    )
}

/// `{"mapped": v}` or `{"unmapped": name key}`.
pub(crate) fn stated_json(stated: &Stated) -> Json {
    match stated {
        Stated::Mapped(value) => json!({ "mapped": value }),
        Stated::Unmapped(key) => json!({ "unmapped": key }),
    }
}

pub(crate) fn stated_from(json: &Json) -> Option<Stated> {
    if let Some(value) = json.get("mapped") {
        Some(Stated::Mapped(value.clone()))
    } else {
        json.get("unmapped")
            .and_then(Json::as_str)
            .map(|k| Stated::Unmapped(k.to_owned()))
    }
}

/// What one answer states for a key: `{stated, value}` or a list of them. A model value that maps
/// is the statement; otherwise the stated text counts as its own (unmapped) value. Two different
/// values state nothing.
fn statement_of(definition: &Definition, entry: &Json) -> Option<Stated> {
    let entries: Vec<&Json> = match entry {
        Json::Array(items) => items.iter().collect(),
        Json::Null => Vec::new(),
        other => vec![other],
    };
    let mut statements: Vec<Stated> = Vec::new();
    for entry in entries {
        let mapped = entry
            .get("value")
            .filter(|v| !v.is_null())
            .and_then(|v| map_output(definition, v).value().cloned());
        let stated = match mapped {
            Some(value) => Stated::Mapped(value),
            None => match entry.get("stated").and_then(Json::as_str) {
                Some(text) if !text.trim().is_empty() => {
                    Stated::Unmapped(psm_core::mechanisms::name_key::name_key(text))
                }
                _ => continue,
            },
        };
        statements.push(stated);
    }
    let first = statements.first()?.clone();
    statements
        .iter()
        .all(|s| same_statement(definition, &first, s))
        .then_some(first)
}

fn statements(definitions: &[Definition], answer: &Json, field: &str) -> Map<String, Json> {
    let mut out = Map::new();
    for definition in definitions {
        if let Some(stated) = answer
            .get(field)
            .and_then(|m| m.get(&definition.key))
            .and_then(|entry| statement_of(definition, entry))
        {
            out.insert(definition.key.clone(), stated_json(&stated));
        }
    }
    out
}

/// Photo reads `{key: {value, sure}}`, each value mapped by MC-A2 (a failed mapping is dropped).
fn reads(definitions: &[Definition], answer: &Json) -> Map<String, Json> {
    let mut out = Map::new();
    for definition in definitions {
        let Some(entry) = answer.get("reads").and_then(|m| m.get(&definition.key)) else {
            continue;
        };
        if let Mapped::Value { value, .. } =
            map_output(definition, entry.get("value").unwrap_or(&Json::Null))
        {
            let sure = entry.get("sure").and_then(Json::as_bool).unwrap_or(false);
            out.insert(
                definition.key.clone(),
                json!({ "value": value, "sure": sure }),
            );
        }
    }
    out
}

pub(crate) fn display(value: &Json) -> String {
    match value {
        Json::String(s) => s.clone(),
        Json::Array(items) => items.iter().map(display).collect::<Vec<_>>().join(", "),
        Json::Object(map) if map.contains_key("value") && map.contains_key("unit") => {
            format!("{} {}", display(&map["value"]), display(&map["unit"]))
        }
        other => other.to_string(),
    }
}

impl Ctx<'_> {
    pub(crate) fn model_request(
        &self,
        stage: Stage,
        unit_key: &str,
        instructions: &str,
        input: Json,
        images: Vec<ResearchImage>,
        listing_images: Vec<String>,
    ) -> CallRequest {
        let settings = self.config.stage(stage);
        let mut input = input;
        if let Some(object) = input.as_object_mut() {
            object.insert("language".into(), json!(self.config.language));
        }
        CallRequest::Model(ModelRequest {
            stage,
            unit_key: unit_key.to_owned(),
            model: settings.model,
            effort: settings.effort,
            language: self.config.language.clone(),
            instructions: instructions.to_owned(),
            input,
            images,
            listing_images,
        })
    }

    /// One call through the gateway, as a unit sees it.
    pub(crate) fn ask(
        &self,
        stage: Stage,
        unit_key: &str,
        request: CallRequest,
    ) -> Result<(Asked, Vec<i64>)> {
        let called = self
            .svc
            .gateway
            .call(self.run.id, stage, unit_key, request)?;
        let asked = match called.outcome {
            GatewayOutcome::Answered(answer) => Asked::Answer(answer),
            GatewayOutcome::NotDetermined(text) => Asked::NotDetermined(text),
            GatewayOutcome::SkipToDecide | GatewayOutcome::EndAtCap => Asked::Work(Work::Skip),
            GatewayOutcome::HoldOffline => Asked::Work(Work::Halt(ExecuteOutcome::HoldOffline)),
            GatewayOutcome::RunFailed { .. } => {
                self.svc.changed(self.run.session_id);
                Asked::Work(Work::Halt(ExecuteOutcome::Failed))
            }
        };
        Ok((asked, called.dropped))
    }

    /// Runs one unit: its committed output if there is one, else its work, committed.
    pub(crate) fn unit(
        &self,
        stage: Stage,
        unit_key: &str,
        identity: Json,
        work: impl FnOnce(&Self) -> Result<Work>,
    ) -> Result<Flow> {
        let fingerprint = fingerprint(stage.number(), &identity)
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        if let Some(row) = self.svc.store.research_transaction(|tx| {
            store::output(tx.conn(), self.run.id, stage.as_str(), unit_key)
        })? {
            return Ok(Flow::Done(Committed {
                stage,
                output: serde_json::from_str(&row.output).unwrap_or(Json::Null),
                fingerprint: row.fingerprint,
            }));
        }
        match work(self)? {
            Work::Commit { output, steps } => {
                self.commit(stage, unit_key, &fingerprint, &output, &steps, None)?;
                Ok(Flow::Done(Committed {
                    stage,
                    output,
                    fingerprint,
                }))
            }
            Work::Skip => Ok(Flow::Skip),
            Work::Halt(outcome) => Ok(Flow::Halt(outcome)),
        }
    }

    /// One transaction: the unit's output, its steps and the stage reached.
    pub(crate) fn commit(
        &self,
        stage: Stage,
        unit_key: &str,
        fingerprint: &str,
        output: &Json,
        steps: &[NewStep],
        reused_from: Option<i64>,
    ) -> Result<()> {
        let now = self.svc.clock.now_ms();
        self.svc.store.research_transaction(|tx| {
            let conn = tx.conn();
            store::insert_output(
                conn,
                &OutputRow {
                    run_id: self.run.id,
                    stage: stage.as_str().to_owned(),
                    unit_key: unit_key.to_owned(),
                    fingerprint: fingerprint.to_owned(),
                    output: output.to_string(),
                    reused_from_run_id: reused_from,
                    committed_at: now,
                },
            )?;
            for step in steps {
                store::insert_step(conn, self.run.id, step, now)?;
            }
            store::set_stage_reached(conn, self.run.id, stage.as_str())
        })?;
        self.svc.changed(self.run.session_id);
        Ok(())
    }

    fn render(&self, photo_id: i64, stage: Stage) -> std::result::Result<ResearchImage, String> {
        self.svc
            .images
            .render(
                photo_id,
                self.run.id,
                self.config.stage(stage).image_long_edge,
            )
            .map_err(|e| match e {
                crate::images::NoSource::NotEligible => PHOTO_REMOVED.to_owned(),
                other => other.to_string(),
            })
    }
}

/// An eligible photo with a research source, as stage units see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UsablePhoto {
    pub id: i64,
    pub classification: Classification,
    pub position: Option<i64>,
    pub revision: i64,
}

/// The run's session photos that are eligible and have a source, main first by position.
pub(crate) fn usable_photos(
    svc: &ResearchService,
    run: &store::RunRow,
) -> Result<Vec<UsablePhoto>> {
    let mut photos = svc.store.research_transaction(|tx| {
        let mut out = Vec::new();
        for photo in tx.photos(run.session_id)? {
            if !eligible(tx, run, photo.id)? || source(tx, photo.id)?.is_none() {
                continue;
            }
            out.push(UsablePhoto {
                id: photo.id,
                classification: photo.classification,
                position: photo.position,
                revision: tx.composition(photo.id)?.revision,
            });
        }
        Ok(out)
    })?;
    photos.sort_by_key(|p| {
        (
            p.classification != Classification::Main,
            p.position.unwrap_or(i64::MAX),
            p.id,
        )
    });
    Ok(photos)
}

pub(crate) fn execute(svc: &ResearchService, run_id: i64) -> Result<ExecuteOutcome> {
    let Some(run) = svc
        .store
        .research_transaction(|tx| store::run(tx.conn(), run_id))?
    else {
        return Ok(ExecuteOutcome::NotRunning);
    };
    if run.state != RunState::Running {
        return Ok(ExecuteOutcome::NotRunning);
    }
    let ctx = Ctx {
        config: ResearchConfig::from_snapshot(run.settings_snapshot.as_deref())?,
        form: FormSnapshot::from_text(run.form_snapshot.as_deref())?,
        svc,
        run,
    };
    if let Some(done) = crate::decide::committed(&ctx)? {
        return crate::decide::finish(&ctx, done);
    }
    let photos = usable_photos(svc, &ctx.run)?;
    let mut stopped = false;
    let mut used: Vec<Committed> = Vec::new();
    macro_rules! flow {
        ($flow:expr) => {
            match $flow {
                Flow::Done(committed) => Some(committed),
                Flow::Skip => {
                    stopped = true;
                    None
                }
                Flow::Halt(outcome) => return Ok(outcome),
            }
        };
    }
    // Stage 1: one unit per main or unreviewed photo.
    let mut observed: Vec<Committed> = Vec::new();
    for photo in photos.iter().filter(|p| {
        matches!(
            p.classification,
            Classification::Main | Classification::Unreviewed
        )
    }) {
        match flow!(observe(&ctx, photo)?) {
            Some(c) => observed.push(c),
            None => break,
        }
    }
    used.extend(observed.iter().cloned());
    // Stage 2: the cover (main position 1).
    let cover = photos
        .iter()
        .find(|p| p.classification == Classification::Main)
        .cloned();
    let mut found: Option<Committed> = None;
    if !stopped && let Some(cover) = &cover {
        found = flow!(find(&ctx, cover)?);
    }
    used.extend(found.iter().cloned());
    // Stage 3: pages in reading order until `max_pages` are obtained.
    let mut pages: Vec<Committed> = Vec::new();
    if !stopped && let Some(found) = &found {
        let candidates: Vec<Candidate> = found.output["candidates"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|c| {
                Some(Candidate {
                    url: c.get("url")?.as_str()?.to_owned(),
                    rank: u32::try_from(c.get("rank")?.as_u64()?).ok()?,
                })
            })
            .collect();
        let trusted = crate::rules::sites::TrustedSites::new(ctx.config.trusted_sites.clone());
        let order = reading_order(
            &candidates,
            &ctx.config.shop_hosts,
            &trusted,
            &ctx.config.own_listing(),
        );
        let mut obtained = 0;
        for page in &order.pages {
            if obtained >= ctx.config.max_pages {
                break;
            }
            match flow!(read(&ctx, &page.url, &page.identity)?) {
                Some(c) => {
                    if c.output["obtained"].as_bool() == Some(true) {
                        obtained += 1;
                    }
                    pages.push(c);
                }
                None => break,
            }
        }
    }
    used.extend(pages.iter().cloned());
    // Stage 4: one unit per obtained page with images.
    if !stopped && let Some(cover) = &cover {
        for page in pages.iter().filter(|p| {
            p.output["obtained"].as_bool() == Some(true)
                && p.output["images"].as_array().is_some_and(|i| !i.is_empty())
        }) {
            match flow!(compare(&ctx, cover, page)?) {
                Some(c) => used.push(c),
                None => break,
            }
        }
    }
    // Stage 5: one unit per photo whose observe output shows a marking.
    if !stopped {
        for observe in observed
            .iter()
            .filter(|o| o.output["shows_marking"].as_bool() == Some(true))
        {
            let Some(photo) = photos
                .iter()
                .find(|p| Some(p.id) == observe.output["photo_id"].as_i64())
            else {
                continue;
            };
            match flow!(labels(&ctx, photo)?) {
                Some(c) => used.push(c),
                None => break,
            }
        }
    }
    // Stage 6: one unit over the condition photos (none: no call).
    let condition_photos: Vec<&UsablePhoto> = photos
        .iter()
        .filter(|p| p.classification == Classification::Condition)
        .collect();
    if !stopped
        && !condition_photos.is_empty()
        && let Some(c) = flow!(condition(&ctx, &condition_photos)?)
    {
        used.push(c);
    }
    // Stage 7.
    crate::decide::decide(&ctx, &used, stopped)
}

fn observe(ctx: &Ctx<'_>, photo: &UsablePhoto) -> Result<Flow> {
    let stage = Stage::Observe;
    let settings = ctx.config.stage(stage);
    let unit_key = format!("photo:{}", photo.id);
    let identity = json!({
        "photo_id": photo.id,
        "composition_rev": photo.revision,
        "long_edge": settings.image_long_edge,
        "model": settings.model,
        "effort": settings.effort,
        "instructions_version": ctx.config.instructions_version,
        "language": ctx.config.language,
    });
    let visible = fields_of(ctx, &[FieldClass::Visible]);
    ctx.unit(stage, &unit_key, identity, |ctx| {
        let mut looked = step(
            stage,
            StepKind::Looked,
            format!("Looked at photo {}", photo.id),
        );
        looked.photo_ids = vec![photo.id];
        let not_determined = |text: String, looked: NewStep| Work::Commit {
            output: json!({
                "photo_id": photo.id,
                "reads": {},
                "shows_marking": false,
                "not_determined": text,
                "files": [],
            }),
            steps: vec![NewStep {
                summary: format!("Could not read photo {}", photo.id),
                detail: text,
                ..looked
            }],
        };
        let image = match ctx.render(photo.id, stage) {
            Ok(image) => image,
            Err(why) => return Ok(not_determined(why, looked)),
        };
        let request = ctx.model_request(
            stage,
            &unit_key,
            OBSERVE,
            json!({ "fields": visible.iter().map(descriptor).collect::<Vec<_>>() }),
            vec![image.clone()],
            Vec::new(),
        );
        let (asked, _) = ctx.ask(stage, &unit_key, request)?;
        let output = match asked {
            Asked::Answer(Answer::Model(reply)) => reply.output,
            Asked::Answer(_) => Json::Null,
            Asked::NotDetermined(text) => return Ok(not_determined(text, looked)),
            Asked::Work(work) => return Ok(work),
        };
        let reads = reads(&visible, &output);
        let shows_marking = output["shows_marking"].as_bool().unwrap_or(false);
        looked.field_keys = reads.keys().cloned().collect();
        looked.detail = reads
            .iter()
            .map(|(k, v)| format!("{k}: {}", display(&v["value"])))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Work::Commit {
            output: json!({
                "photo_id": photo.id,
                "composition_rev": photo.revision,
                "image": image.sha256(),
                "reads": reads,
                "shows_marking": shows_marking,
                "files": [image.sha256()],
            }),
            steps: vec![looked],
        })
    })
}

fn find(ctx: &Ctx<'_>, cover: &UsablePhoto) -> Result<Flow> {
    let stage = Stage::Find;
    let provider = ctx
        .svc
        .providers
        .search
        .as_ref()
        .map(|p| p.id())
        .unwrap_or_default();
    let identity = json!({
        "cover_photo_id": cover.id,
        "composition_rev": cover.revision,
        "provider": provider,
    });
    ctx.unit(stage, "cover", identity, |ctx| {
        let mut searched = step(stage, StepKind::Searched, "Searched for matching pages");
        searched.photo_ids = vec![cover.id];
        let image = match ctx.render(cover.id, stage) {
            Ok(image) => image,
            Err(why) => {
                searched.detail = why;
                return Ok(Work::Commit {
                    output: json!({"cover": cover.id, "candidates": [], "files": []}),
                    steps: vec![searched],
                });
            }
        };
        let (asked, _) = ctx.ask(stage, "cover", CallRequest::Search(image.clone()))?;
        let candidates = match asked {
            Asked::Answer(Answer::Search(reply)) => reply.candidates,
            Asked::Answer(_) => Vec::new(),
            Asked::NotDetermined(text) => {
                searched.detail = text;
                Vec::new()
            }
            Asked::Work(work) => return Ok(work),
        };
        searched.summary = format!("Found {} candidate pages", candidates.len());
        let mut steps = vec![searched];
        let trusted = crate::rules::sites::TrustedSites::new(ctx.config.trusted_sites.clone());
        let order = reading_order(
            &candidates
                .iter()
                .map(|c| Candidate {
                    url: c.url.clone(),
                    rank: c.rank,
                })
                .collect::<Vec<_>>(),
            &ctx.config.shop_hosts,
            &trusted,
            &ctx.config.own_listing(),
        );
        for own in &order.own_listing {
            let mut own_step = step(Stage::Read, StepKind::Read, "this item's own listing");
            own_step.source_url = Some(own.url.clone());
            steps.push(own_step);
        }
        Ok(Work::Commit {
            output: json!({
                "cover": cover.id,
                "composition_rev": cover.revision,
                "image": image.sha256(),
                "candidates": candidates,
                "files": [image.sha256()],
            }),
            steps,
        })
    })
}

fn read(ctx: &Ctx<'_>, url: &str, identity_u: &str) -> Result<Flow> {
    let stage = Stage::Read;
    let settings = ctx.config.stage(stage);
    let researched = fields_of(
        ctx,
        &[
            FieldClass::Attribution,
            FieldClass::Measured,
            FieldClass::Visible,
        ],
    );
    let route = ctx
        .svc
        .providers
        .reader
        .as_ref()
        .map(|p| p.id())
        .unwrap_or_default();
    let identity = json!({
        "page": identity_u,
        "route": route,
        "model": settings.model,
        "effort": settings.effort,
        "instructions_version": ctx.config.instructions_version,
        "keys": researched.iter().map(descriptor).collect::<Vec<_>>(),
    });
    ctx.unit(stage, identity_u, identity, |ctx| {
        let mut read_step = step(stage, StepKind::Read, "could not be read");
        read_step.source_url = Some(url.to_owned());
        let unread = |read_step: NewStep| Work::Commit {
            output: json!({
                "url": url,
                "unit": identity_u,
                "obtained": false,
                "statements": {},
                "images": [],
                "files": [],
            }),
            steps: vec![read_step],
        };
        if ctx.svc.providers.reader.is_none() {
            return Ok(unread(read_step));
        }
        let (asked, _) = ctx.ask(
            stage,
            identity_u,
            CallRequest::Read {
                url: url.to_owned(),
            },
        )?;
        let (page, images) = match asked {
            Asked::Answer(Answer::Page {
                reply,
                image_sha256,
            }) => (reply, image_sha256),
            Asked::Answer(_) | Asked::NotDetermined(_) => return Ok(unread(read_step)),
            Asked::Work(work) => return Ok(work),
        };
        if !page.obtained {
            return Ok(unread(read_step));
        }
        let read_url = page.final_url.clone().unwrap_or_else(|| url.to_owned());
        let trusted = crate::rules::sites::TrustedSites::new(ctx.config.trusted_sites.clone());
        let site = site_of(&read_url, &ctx.config.shop_hosts, &trusted);
        let page_u = page_identity(&read_url).unwrap_or_else(|| identity_u.to_owned());
        let request = ctx.model_request(
            stage,
            identity_u,
            READ,
            json!({
                "page": {"url": read_url, "title": page.title, "text": page.text},
                "fields": researched.iter().map(descriptor).collect::<Vec<_>>(),
            }),
            Vec::new(),
            Vec::new(),
        );
        let (asked, _) = ctx.ask(stage, identity_u, request)?;
        let (statements, not_determined) = match asked {
            Asked::Answer(Answer::Model(reply)) => {
                (statements(&researched, &reply.output, "keys"), None)
            }
            Asked::Answer(_) => (Map::new(), None),
            Asked::NotDetermined(text) => (Map::new(), Some(text)),
            Asked::Work(work) => return Ok(work),
        };
        read_step.summary = if page.title.is_empty() {
            format!("Read {read_url}")
        } else {
            format!("Read {}", page.title)
        };
        read_step.source_url = Some(read_url.clone());
        read_step.source_title = Some(page.title.clone());
        read_step.field_keys = statements.keys().cloned().collect();
        if let Some(text) = &not_determined {
            read_step.detail = text.clone();
        }
        Ok(Work::Commit {
            output: json!({
                "url": url,
                "final_url": page.final_url,
                "unit": identity_u,
                "page": page_u,
                "title": page.title,
                "obtained": true,
                "site": site.map(|s| json!({"class": s.class, "identity": s.identity})),
                "statements": statements,
                "images": images,
                "not_determined": not_determined,
                "files": images,
            }),
            steps: vec![read_step],
        })
    })
}

fn compare(ctx: &Ctx<'_>, cover: &UsablePhoto, page: &Committed) -> Result<Flow> {
    let stage = Stage::Compare;
    let unit_key = page.output["unit"].as_str().unwrap_or_default().to_owned();
    let images: Vec<String> = page.output["images"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.as_str().map(str::to_owned))
        .collect();
    let identity = json!({
        "page": unit_key,
        "images": images,
        "cover": [cover.id, cover.revision],
    });
    ctx.unit(stage, &unit_key.clone(), identity, |ctx| {
        let title = page.output["title"].as_str().unwrap_or_default().to_owned();
        let mut compared = step(stage, StepKind::Compared, format!("Compared with {title}"));
        compared.photo_ids = vec![cover.id];
        compared.source_url = page.output["final_url"]
            .as_str()
            .or(page.output["url"].as_str())
            .map(str::to_owned);
        compared.source_title = Some(title.clone());
        let image = match ctx.render(cover.id, stage) {
            Ok(image) => image,
            Err(why) => {
                compared.detail = why;
                return Ok(Work::Commit {
                    output: json!({"page": page.output["page"], "unit": unit_key, "verdict": null, "files": []}),
                    steps: vec![compared],
                });
            }
        };
        let request = ctx.model_request(
            stage,
            &unit_key,
            COMPARE,
            json!({ "page": {"url": compared.source_url, "title": title} }),
            vec![image.clone()],
            images.clone(),
        );
        let (asked, _) = ctx.ask(stage, &unit_key, request)?;
        let verdict = match asked {
            Asked::Answer(Answer::Model(reply)) => reply.output["verdict"]
                .as_str()
                .filter(|v| matches!(*v, "match" | "similar" | "different"))
                .map(str::to_owned),
            Asked::Answer(_) => None,
            Asked::NotDetermined(text) => {
                compared.detail = text;
                None
            }
            Asked::Work(work) => return Ok(work),
        };
        compared.summary = match &verdict {
            Some(v) => format!("Compared with {title}: {v}"),
            None => format!("Compared with {title}: not determined"),
        };
        Ok(Work::Commit {
            output: json!({
                "page": page.output["page"],
                "unit": unit_key,
                "verdict": verdict,
                "files": [image.sha256()],
            }),
            steps: vec![compared],
        })
    })
}

fn labels(ctx: &Ctx<'_>, photo: &UsablePhoto) -> Result<Flow> {
    let stage = Stage::Labels;
    let settings = ctx.config.stage(stage);
    let unit_key = format!("photo:{}", photo.id);
    let identity = json!({
        "photo_id": photo.id,
        "composition_rev": photo.revision,
        "model": settings.model,
        "effort": settings.effort,
        "instructions_version": ctx.config.instructions_version,
    });
    let keys = fields_of(ctx, &[FieldClass::Attribution, FieldClass::Visible]);
    ctx.unit(stage, &unit_key, identity, |ctx| {
        let mut looked = step(
            stage,
            StepKind::Looked,
            format!("Read the marking on photo {}", photo.id),
        );
        looked.photo_ids = vec![photo.id];
        let image = match ctx.render(photo.id, stage) {
            Ok(image) => image,
            Err(why) => {
                looked.detail = why;
                return Ok(Work::Commit {
                    output: json!({"photo_id": photo.id, "statements": {}, "files": []}),
                    steps: vec![looked],
                });
            }
        };
        let request = ctx.model_request(
            stage,
            &unit_key,
            LABELS,
            json!({ "fields": keys.iter().map(descriptor).collect::<Vec<_>>() }),
            vec![image.clone()],
            Vec::new(),
        );
        let (asked, _) = ctx.ask(stage, &unit_key, request)?;
        let (statements, text) = match asked {
            Asked::Answer(Answer::Model(reply)) => (
                statements(&keys, &reply.output, "labels"),
                reply.output["text"].as_str().unwrap_or_default().to_owned(),
            ),
            Asked::Answer(_) => (Map::new(), String::new()),
            Asked::NotDetermined(text) => (Map::new(), text),
            Asked::Work(work) => return Ok(work),
        };
        looked.field_keys = statements.keys().cloned().collect();
        looked.detail = text.clone();
        Ok(Work::Commit {
            output: json!({
                "photo_id": photo.id,
                "statements": statements,
                "text": text,
                "files": [image.sha256()],
            }),
            steps: vec![looked],
        })
    })
}

fn condition(ctx: &Ctx<'_>, photos: &[&UsablePhoto]) -> Result<Flow> {
    let stage = Stage::Condition;
    let keys = fields_of(ctx, &[FieldClass::Condition]);
    let mut identities: Vec<(i64, i64)> = photos.iter().map(|p| (p.id, p.revision)).collect();
    identities.sort();
    let identity = json!({
        "photos": identities.iter().map(|(id, rev)| json!([id, rev])).collect::<Vec<_>>(),
        "definitions": keys.iter().map(descriptor).collect::<Vec<_>>(),
    });
    ctx.unit(stage, "condition", identity, |ctx| {
        let ids: Vec<i64> = photos.iter().map(|p| p.id).collect();
        let mut looked = step(stage, StepKind::Looked, "Looked at the condition photos");
        looked.photo_ids = ids.clone();
        let mut images = Vec::new();
        for photo in photos {
            if let Ok(image) = ctx.render(photo.id, stage) {
                images.push(image);
            }
        }
        let files: Vec<String> = images.iter().map(|i| i.sha256().to_owned()).collect();
        let request = ctx.model_request(
            stage,
            "condition",
            CONDITION,
            json!({ "fields": keys.iter().map(descriptor).collect::<Vec<_>>() }),
            images,
            Vec::new(),
        );
        let (asked, _) = ctx.ask(stage, "condition", request)?;
        let read = match asked {
            Asked::Answer(Answer::Model(reply)) => reads(&keys, &reply.output),
            Asked::Answer(_) => Map::new(),
            Asked::NotDetermined(text) => {
                looked.detail = text;
                Map::new()
            }
            Asked::Work(work) => return Ok(work),
        };
        looked.field_keys = read.keys().cloned().collect();
        Ok(Work::Commit {
            output: json!({ "photos": ids, "reads": read, "files": files }),
            steps: vec![looked],
        })
    })
}
