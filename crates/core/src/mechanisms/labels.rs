//! The pure rules of labels (book 09C §7, §8; `master_plan_labels.md` §4 Constants, Pure rules,
//! §5 MC-L3.2): request chunking and the display copy of label state. Memory only: nothing here
//! is ever stored (M-L1).

use crate::domain::label::{
    CheckGroup, LabelChange, LabelChangeKind, LabelPlan, LabelState, LabelStatus, LabelTemplate,
    LinkedItem, PlanSummary, Unresolved,
};
use std::{collections::HashMap, ops::Range, time::Duration, time::Instant};

/// At most this many product ids per status request (the provider's maximum).
pub const STATUS_CHUNK: usize = 200;
/// A status answer younger than this is not asked again by a refreshing read.
pub const STATUS_FRESH_MS: u64 = 10_000;
/// L2 (MC-L1.1): rows per page of the Labels box.
pub const LIST_PAGE: u32 = 25;
/// L2 (MC-L1.7): rows per request of Select all (the provider's maximum).
pub const SELECT_ALL_PAGE: u32 = 500;
/// L2 (MC-L1.7): Select all never asks more pages than this.
pub const SELECT_ALL_MAX_PAGES: usize = 200;
/// L2 (MC-L1.6): a remembered unsearched count younger than this is not asked again.
pub const COUNT_FRESH_MS: u64 = 10_000;
/// L2 (MC-L1.1): the search sent to the provider is cut to this many characters.
pub const QUERY_MAX: usize = 100;

/// L3 (MC-L2.1, .6): at most this many products per check request and per run.
pub const RUN_CHUNK: usize = 300;
/// L3 (MC-L2.7): a run that could not be reached is sent again this many more times, at once.
pub const RUN_RETRIES: usize = 2;
/// L3 (MC-L2.5): the long edge of a preview picture, in pixels.
pub const PREVIEW_EDGE: u32 = 1200;
/// L3 (MC-L2.1): the message of a product a check chunk did not find.
pub const NOT_FOUND: &str = "Not found";
/// L3 (MC-L2.13): the name of a label file when nothing usable is left of the given one.
pub const DEFAULT_LABEL_FILE: &str = "labels.pdf";
/// L3 (MC-L2.13): the part of a label file name before `.pdf` is cut to this many characters.
pub const LABEL_STEM_MAX: usize = 120;

/// L3 (MC-L2.1): the check answers of consecutive chunks (each with its index) as one plan:
/// items in chunk order with `position` = the index among the merged items (0, 1, 2, …,
/// independent of the provider's numbering), `unresolved` concatenated, each summary number
/// summed, `sizes` merged where width and height are equal (first-appearance order). An `Err`
/// member is a chunk the provider found none of: its products become `Unresolved` with the
/// message `Not found`.
pub fn merge_plans(mut answers: Vec<(usize, Result<LabelPlan, Vec<Unresolved>>)>) -> LabelPlan {
    answers.sort_by_key(|(index, _)| *index);
    let mut merged = LabelPlan {
        items: Vec::new(),
        unresolved: Vec::new(),
        summary: PlanSummary {
            items: 0,
            ready: 0,
            failing: 0,
            mark_printed: 0,
            reprint: 0,
            no_price: 0,
        },
        sizes: Vec::new(),
    };
    for (_, answer) in answers {
        match answer {
            Ok(plan) => {
                for mut item in plan.items {
                    item.position = merged.items.len();
                    merged.items.push(item);
                }
                merged.unresolved.extend(plan.unresolved);
                let (m, s) = (&mut merged.summary, plan.summary);
                m.items += s.items;
                m.ready += s.ready;
                m.failing += s.failing;
                m.mark_printed += s.mark_printed;
                m.reprint += s.reprint;
                m.no_price += s.no_price;
                for size in plan.sizes {
                    match merged
                        .sizes
                        .iter_mut()
                        .find(|z| z.width_mm == size.width_mm && z.height_mm == size.height_mm)
                    {
                        Some(z) => z.count += size.count,
                        None => merged.sizes.push(size),
                    }
                }
            }
            Err(missing) => merged
                .unresolved
                .extend(missing.into_iter().map(|u| Unresolved {
                    message: NOT_FOUND.into(),
                    ..u
                })),
        }
    }
    merged
}

/// L3 (MC-L2.2, .4): the check's items grouped by template id in first-appearance order, items
/// in plan order, the items without a template in one last group. A group's product types are
/// its template's in `templates` (empty when the id is not there).
pub fn group_check(plan: &LabelPlan, templates: &[LabelTemplate]) -> Vec<CheckGroup> {
    let mut groups: Vec<CheckGroup> = Vec::new();
    let mut without: Vec<_> = Vec::new();
    for item in &plan.items {
        let Some(template) = &item.template else {
            without.push(item.clone());
            continue;
        };
        match groups
            .iter_mut()
            .find(|g| g.template.as_ref().is_some_and(|t| t.id == template.id))
        {
            Some(group) => group.items.push(item.clone()),
            None => groups.push(CheckGroup {
                template: Some(template.clone()),
                product_types: templates
                    .iter()
                    .find(|t| t.id == template.id)
                    .map(|t| t.product_types.clone())
                    .unwrap_or_default(),
                items: vec![item.clone()],
            }),
        }
    }
    if !without.is_empty() {
        groups.push(CheckGroup {
            template: None,
            product_types: Vec::new(),
            items: without,
        });
    }
    groups
}

/// Splits `name` at a final `.pdf` (any case): the stem and that suffix, or the whole and none.
fn split_pdf(name: &str) -> (&str, Option<&str>) {
    let n = name.len();
    if n >= 4 && name.is_char_boundary(n - 4) && name[n - 4..].eq_ignore_ascii_case(".pdf") {
        (&name[..n - 4], Some(&name[n - 4..]))
    } else {
        (name, None)
    }
}

/// L3 (MC-L2.13): the name a label file is saved under: the text after the last `/` or `\`,
/// without control characters and `:`, leading dots and surrounding spaces removed, `.pdf` added
/// unless it ends with it (any case), `labels.pdf` when nothing is left, the part before `.pdf`
/// cut to 120 characters.
pub fn label_file_name(raw: &str) -> String {
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: String = last
        .chars()
        .filter(|c| !c.is_control() && *c != ':')
        .collect();
    let (stem, suffix) = split_pdf(clean.trim());
    let stem = stem.trim_start_matches('.').trim();
    if stem.is_empty() {
        return DEFAULT_LABEL_FILE.into();
    }
    let stem: String = stem.chars().take(LABEL_STEM_MAX).collect();
    format!("{}{}", stem.trim_end(), suffix.unwrap_or(".pdf"))
}

/// L3 (MC-L2.13): `<stem> (<n>).pdf`, the name tried when `name` is taken.
pub fn numbered_name(name: &str, n: u32) -> String {
    let (stem, _) = split_pdf(name);
    format!("{stem} ({n}).pdf")
}

/// L2 (MC-L1.3): the local item a provider product is matched to, as an index into `items`
/// (each item with its link's provider product id): (i) the item whose link names the product
/// (the lowest item id when several do); else (ii) the item whose SKU equals `sku` exactly
/// (case-sensitive); else none.
pub fn match_product(
    product_id: &str,
    sku: Option<&str>,
    items: &[(LinkedItem, Option<String>)],
) -> Option<usize> {
    let linked = items
        .iter()
        .enumerate()
        .filter(|(_, (_, product))| product.as_deref() == Some(product_id))
        .min_by_key(|(_, (item, _))| item.item_id)
        .map(|(i, _)| i);
    linked.or_else(|| {
        let sku = sku?;
        items.iter().position(|(item, _)| item.sku == sku)
    })
}

/// Consecutive ranges covering `0..n`, each `size` long except the last, which is shorter;
/// none for `n = 0`.
pub fn chunks(n: usize, size: usize) -> Vec<Range<usize>> {
    let size = size.max(1);
    (0..n)
        .step_by(size)
        .map(|start| start..(start + size).min(n))
        .collect()
}

#[derive(Debug, Clone, Default)]
struct Entry {
    /// `None` = held with no state (the provider answered that it does not know the product).
    state: Option<LabelState>,
    /// The sequence number of the last event applied to this product (0 = none).
    event_seq: u64,
    /// When a status or list answer was last applied (monotonic).
    answered_at: Option<Instant>,
}

/// The display copy (MC-L3.2): product id → label state, the sequence number of the last event
/// applied to it, and when it was last answered.
#[derive(Debug, Default)]
pub struct LabelCopy {
    seq: u64,
    /// Answers started before this sequence were asked before the copy was emptied.
    floor: u64,
    entries: HashMap<String, Entry>,
}

impl LabelCopy {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current sequence, taken before a request whose answer will be applied.
    pub fn start(&self) -> u64 {
        self.seq
    }

    /// Applies one answer of a request started at `started`: stored (with no state for `None`,
    /// D10) only when no event was applied to that product after `started` — an event that
    /// arrived while the request was in flight wins over the older answer.
    pub fn apply_answer(
        &mut self,
        started: u64,
        product_id: &str,
        state: Option<LabelState>,
        now: Instant,
    ) {
        if started < self.floor {
            return;
        }
        let entry = self.entries.entry(product_id.to_owned()).or_default();
        if entry.event_seq > started {
            return;
        }
        entry.state = state;
        entry.answered_at = Some(now);
    }

    /// Applies one label event: `Printed` → printed at the event's time with a price and no
    /// reset; `Reset` → not printed with the reason, the price as it was (unknown without an
    /// entry). Creates the entry when there is none and raises the sequence.
    pub fn apply_event(&mut self, change: &LabelChange) {
        self.seq += 1;
        let seq = self.seq;
        let entry = self.entries.entry(change.product_id.clone()).or_default();
        entry.state = Some(match change.kind {
            LabelChangeKind::Printed { .. } => LabelState {
                status: LabelStatus::Printed,
                printed_at: Some(change.at),
                has_price: Some(true),
                reset: None,
            },
            LabelChangeKind::Reset { reason } => LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: entry.state.and_then(|s| s.has_price),
                reset: reason,
            },
        });
        entry.event_seq = seq;
    }

    /// The state held for the product.
    pub fn get(&self, product_id: &str) -> Option<LabelState> {
        self.entries.get(product_id).and_then(|e| e.state)
    }

    /// Whether the product was answered less than `max_age` before `now`.
    pub fn fresh(&self, product_id: &str, now: Instant, max_age: Duration) -> bool {
        self.entries
            .get(product_id)
            .and_then(|e| e.answered_at)
            .is_some_and(|at| now.saturating_duration_since(at) < max_age)
    }

    /// Empties the copy; answers to requests started before this are not applied.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.seq += 1;
        self.floor = self.seq;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::label::{CheckOutcome, LabelSize, PlanItem, PlanTemplate, ResetReason};

    fn template(id: &str, w: f64, h: f64) -> PlanTemplate {
        PlanTemplate {
            id: id.into(),
            name: format!("T {id}"),
            width_mm: w,
            height_mm: h,
        }
    }

    fn plan_item(position: usize, id: &str, template: Option<PlanTemplate>) -> PlanItem {
        PlanItem {
            position,
            product_id: id.into(),
            sku: Some(format!("SKU-{id}")),
            title: Some(format!("Title {id}")),
            product_type: None,
            template,
            price_minor: Some(100),
            status: LabelStatus::NotPrinted,
            printed_at: None,
            outcome: CheckOutcome::MarkPrinted,
            issues: vec![],
            notes: vec![],
        }
    }

    fn summary(n: [i64; 6]) -> PlanSummary {
        PlanSummary {
            items: n[0],
            ready: n[1],
            failing: n[2],
            mark_printed: n[3],
            reprint: n[4],
            no_price: n[5],
        }
    }

    fn size(w: f64, h: f64, count: i64) -> LabelSize {
        LabelSize {
            width_mm: w,
            height_mm: h,
            count,
        }
    }

    fn unresolved(id: &str, message: &str) -> Unresolved {
        Unresolved {
            product_id: Some(id.into()),
            sku: None,
            message: message.into(),
        }
    }

    fn ids(plan: &LabelPlan) -> Vec<(usize, String)> {
        plan.items
            .iter()
            .map(|i| (i.position, i.product_id.clone()))
            .collect()
    }

    #[test]
    fn merge_two_plans_renumbers_sums_and_merges_sizes() {
        let a = template("a", 70.0, 100.0);
        let b = template("b", 50.0, 30.0);
        let first = LabelPlan {
            items: vec![
                plan_item(0, "1", Some(a.clone())),
                plan_item(1, "2", Some(b.clone())),
            ],
            unresolved: vec![unresolved("9", "No product with id '9'.")],
            summary: summary([2, 2, 0, 2, 0, 0]),
            sizes: vec![size(70.0, 100.0, 1), size(50.0, 30.0, 1)],
        };
        let second = LabelPlan {
            items: vec![plan_item(0, "3", Some(b)), plan_item(1, "4", Some(a))],
            unresolved: vec![unresolved("8", "Gone.")],
            summary: summary([2, 1, 1, 0, 1, 0]),
            sizes: vec![size(50.0, 30.0, 1), size(70.0, 100.0, 1)],
        };
        let merged = merge_plans(vec![(1, Ok(second)), (0, Ok(first))]);
        assert_eq!(
            ids(&merged),
            vec![
                (0, "1".to_owned()),
                (1, "2".to_owned()),
                (2, "3".to_owned()),
                (3, "4".to_owned())
            ]
        );
        assert_eq!(merged.summary, summary([4, 3, 1, 2, 1, 0]));
        assert_eq!(
            merged.sizes,
            vec![size(70.0, 100.0, 2), size(50.0, 30.0, 2)]
        );
        assert_eq!(
            merged.unresolved,
            vec![
                unresolved("9", "No product with id '9'."),
                unresolved("8", "Gone.")
            ]
        );
    }

    #[test]
    fn merge_with_a_chunk_found_nothing_lists_its_products_not_found() {
        let first = LabelPlan {
            items: vec![plan_item(0, "1", None)],
            unresolved: vec![],
            summary: summary([1, 1, 0, 1, 0, 0]),
            sizes: vec![],
        };
        let merged = merge_plans(vec![
            (0, Ok(first)),
            (1, Err(vec![unresolved("301", "anything")])),
        ]);
        assert_eq!(ids(&merged), vec![(0, "1".to_owned())]);
        assert_eq!(merged.unresolved, vec![unresolved("301", "Not found")]);
        assert_eq!(merged.summary, summary([1, 1, 0, 1, 0, 0]));
    }

    #[test]
    fn merge_positions_ignore_unresolved_entries_and_provider_numbering() {
        // The provider numbers chunk 1's items 0 and 2 (position 1 was the unresolved one).
        let first = LabelPlan {
            items: vec![plan_item(0, "1", None), plan_item(2, "3", None)],
            unresolved: vec![unresolved("2", "Missing.")],
            summary: summary([2, 2, 0, 2, 0, 0]),
            sizes: vec![],
        };
        let second = LabelPlan {
            items: vec![plan_item(0, "4", None)],
            unresolved: vec![],
            summary: summary([1, 1, 0, 1, 0, 0]),
            sizes: vec![],
        };
        let merged = merge_plans(vec![(0, Ok(first)), (1, Ok(second))]);
        assert_eq!(
            ids(&merged),
            vec![
                (0, "1".to_owned()),
                (1, "3".to_owned()),
                (2, "4".to_owned())
            ]
        );
    }

    #[test]
    fn groups_by_template_in_first_appearance_with_no_template_last() {
        let a = template("a", 70.0, 100.0);
        let b = template("b", 50.0, 30.0);
        let plan = LabelPlan {
            items: vec![
                plan_item(0, "1", None),
                plan_item(1, "2", Some(b.clone())),
                plan_item(2, "3", Some(a.clone())),
                plan_item(3, "4", Some(b.clone())),
            ],
            unresolved: vec![],
            summary: summary([4, 4, 0, 4, 0, 0]),
            sizes: vec![],
        };
        let templates = vec![LabelTemplate {
            id: "b".into(),
            name: "T b".into(),
            width_mm: 50.0,
            height_mm: 30.0,
            is_default: false,
            product_types: vec!["Tables".into(), "Chairs".into()],
        }];
        let groups = group_check(&plan, &templates);
        let shape: Vec<(Option<String>, Vec<String>, Vec<String>)> = groups
            .iter()
            .map(|g| {
                (
                    g.template.as_ref().map(|t| t.id.clone()),
                    g.product_types.clone(),
                    g.items.iter().map(|i| i.product_id.clone()).collect(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            vec![
                (
                    Some("b".to_owned()),
                    vec!["Tables".to_owned(), "Chairs".to_owned()],
                    vec!["2".to_owned(), "4".to_owned()]
                ),
                (Some("a".to_owned()), vec![], vec!["3".to_owned()]),
                (None, vec![], vec!["1".to_owned()]),
            ]
        );
    }

    #[test]
    fn label_file_names() {
        assert_eq!(label_file_name("a/b/c.pdf"), "c.pdf");
        assert_eq!(label_file_name("..\\x.PDF"), "x.PDF");
        assert_eq!(label_file_name("report"), "report.pdf");
        assert_eq!(label_file_name(""), "labels.pdf");
        assert_eq!(label_file_name(".pdf"), "labels.pdf");
        assert_eq!(label_file_name("../"), "labels.pdf");
        assert_eq!(label_file_name("a:b\u{7}.pdf"), "ab.pdf");
        let long = "x".repeat(200);
        assert_eq!(
            label_file_name(&format!("{long}.pdf")),
            format!("{}.pdf", "x".repeat(120))
        );
    }

    #[test]
    fn numbered_names() {
        assert_eq!(numbered_name("a.pdf", 2), "a (2).pdf");
        assert_eq!(
            numbered_name("labels-2026-11-20.pdf", 3),
            "labels-2026-11-20 (3).pdf"
        );
    }

    fn printed(at: i64) -> LabelState {
        LabelState {
            status: LabelStatus::Printed,
            printed_at: Some(at),
            has_price: Some(true),
            reset: None,
        }
    }

    fn missing_tag() -> LabelState {
        LabelState {
            status: LabelStatus::NotPrinted,
            printed_at: None,
            has_price: Some(true),
            reset: None,
        }
    }

    fn change(product_id: &str, at: i64, kind: LabelChangeKind) -> LabelChange {
        LabelChange {
            product_id: product_id.into(),
            at,
            kind,
        }
    }

    fn item(item_id: i64, sku: &str) -> LinkedItem {
        LinkedItem {
            item_id,
            sku: sku.into(),
            external_id: None,
            session_id: Some(item_id * 10),
        }
    }

    #[test]
    fn match_by_link() {
        let items = vec![(item(1, "A"), None), (item(2, "B"), Some("p7".to_owned()))];
        assert_eq!(match_product("p7", Some("X"), &items), Some(1));
    }

    #[test]
    fn match_by_sku() {
        let items = vec![(item(1, "A"), None), (item(2, "AB-1"), None)];
        assert_eq!(match_product("p7", Some("AB-1"), &items), Some(1));
    }

    #[test]
    fn the_link_wins_over_the_sku() {
        let items = vec![
            (item(1, "AB-1"), None),
            (item(2, "B"), Some("p7".to_owned())),
        ];
        assert_eq!(match_product("p7", Some("AB-1"), &items), Some(1));
    }

    #[test]
    fn two_items_linked_to_one_product_match_the_lowest_item_id() {
        let items = vec![
            (item(5, "A"), Some("p7".to_owned())),
            (item(3, "B"), Some("p7".to_owned())),
            (item(9, "C"), Some("p7".to_owned())),
        ];
        assert_eq!(match_product("p7", None, &items), Some(1));
    }

    #[test]
    fn no_sku_and_no_link_match_nothing() {
        let items = vec![(item(1, "A"), Some("p1".to_owned()))];
        assert_eq!(match_product("p7", None, &items), None);
    }

    #[test]
    fn a_sku_differing_only_in_case_matches_nothing() {
        let items = vec![(item(1, "AB-1"), None)];
        assert_eq!(match_product("p7", Some("ab-1"), &items), None);
    }

    #[test]
    fn chunks_of_nothing_is_none() {
        assert_eq!(chunks(0, 200), Vec::<Range<usize>>::new());
    }

    #[test]
    fn chunks_of_exactly_one_size() {
        assert_eq!(chunks(200, 200), vec![0..200]);
    }

    #[test]
    fn chunks_with_a_shorter_last_one() {
        assert_eq!(chunks(201, 200), vec![0..200, 200..201]);
    }

    #[test]
    fn an_answer_older_than_an_event_for_the_same_product_is_ignored() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let started = copy.start();
        copy.apply_event(&change(
            "p1",
            5,
            LabelChangeKind::Printed { price_minor: None },
        ));
        copy.apply_answer(started, "p1", Some(missing_tag()), now);
        assert_eq!(copy.get("p1"), Some(printed(5)));
    }

    #[test]
    fn an_answer_for_another_product_than_the_event_is_stored() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let started = copy.start();
        copy.apply_event(&change(
            "p1",
            5,
            LabelChangeKind::Printed { price_minor: None },
        ));
        copy.apply_answer(started, "p2", Some(missing_tag()), now);
        assert_eq!(copy.get("p2"), Some(missing_tag()));
    }

    #[test]
    fn reset_keeps_the_price_it_had() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let mut no_price = printed(1);
        no_price.has_price = Some(false);
        copy.apply_answer(copy.start(), "p1", Some(no_price), now);
        copy.apply_event(&change(
            "p1",
            9,
            LabelChangeKind::Reset {
                reason: Some(ResetReason::PriceChanged),
            },
        ));
        assert_eq!(
            copy.get("p1"),
            Some(LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: Some(false),
                reset: Some(ResetReason::PriceChanged),
            })
        );
    }

    #[test]
    fn reset_without_an_entry_creates_one_with_the_price_unknown() {
        let mut copy = LabelCopy::new();
        copy.apply_event(&change(
            "p9",
            9,
            LabelChangeKind::Reset {
                reason: Some(ResetReason::SkuChanged),
            },
        ));
        assert_eq!(
            copy.get("p9"),
            Some(LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: None,
                reset: Some(ResetReason::SkuChanged),
            })
        );
    }

    #[test]
    fn fresh_within_the_age_and_not_after_it_nor_without_an_entry() {
        let now = Instant::now();
        let age = Duration::from_millis(STATUS_FRESH_MS);
        let mut copy = LabelCopy::new();
        copy.apply_answer(copy.start(), "p1", Some(missing_tag()), now);
        assert!(copy.fresh("p1", now + Duration::from_secs(9), age));
        assert!(!copy.fresh("p1", now + Duration::from_secs(11), age));
        assert!(!copy.fresh("p2", now, age));
    }
}
