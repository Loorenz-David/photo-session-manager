//! MC-A4: value and confidence of one field from recorded evidence, as a total function. A model
//! never sets a confidence; nothing here takes one from its inputs.

use std::collections::BTreeSet;

use psm_core::domain::attribute::{AiConfidence, AttributeGroup, CoreType, Definition};
use psm_core::mechanisms::sameness::same;
use serde_json::Value as Json;

use crate::domain::{FieldClass, LabelItem, ListingItem, PhotoRead, SiteClass, Verdict};
use crate::rules::value_map::{Stated, same_statement};

/// The outcome for one field: the written value and confidence (none when not determined), the
/// page identities supporting the written value at the decided level (in the order given), and
/// the other values, for the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub written: Option<(Json, AiConfidence)>,
    pub sources: Vec<String>,
    pub named: Vec<Stated>,
}

impl Decision {
    fn not_determined(named: Vec<Stated>) -> Self {
        Self {
            written: None,
            sources: Vec::new(),
            named,
        }
    }

    pub fn confidence(&self) -> Option<AiConfidence> {
        self.written.as_ref().map(|(_, c)| *c)
    }

    pub fn value(&self) -> Option<&Json> {
        self.written.as_ref().map(|(v, _)| v)
    }
}

/// MC-A4 field class, first match: never-written, condition, attribution, measured, visible.
pub fn field_class(
    definition: &Definition,
    never_written: bool,
    attribution_keys: &[String],
) -> FieldClass {
    if never_written {
        FieldClass::NeverWritten
    } else if definition.group == AttributeGroup::Condition {
        FieldClass::Condition
    } else if attribution_keys.contains(&definition.key) {
        FieldClass::Attribution
    } else if matches!(definition.core_type, CoreType::Dimension | CoreType::Weight) {
        FieldClass::Measured
    } else {
        FieldClass::Visible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Low,
    Medium,
    High,
}

impl Level {
    fn confidence(self) -> AiConfidence {
        match self {
            Level::Low => AiConfidence::Low,
            Level::Medium => AiConfidence::Medium,
            Level::High => AiConfidence::High,
        }
    }
}

/// The evidence for one value.
struct Group {
    stated: Stated,
    label_photos: BTreeSet<i64>,
    shop_match_pages: Vec<String>,
    medium_ids: BTreeSet<String>,
    medium_pages: Vec<String>,
    low_ids: BTreeSet<String>,
    low_pages: Vec<String>,
}

impl Group {
    fn new(stated: Stated) -> Self {
        Self {
            stated,
            label_photos: BTreeSet::new(),
            shop_match_pages: Vec::new(),
            medium_ids: BTreeSet::new(),
            medium_pages: Vec::new(),
            low_ids: BTreeSet::new(),
            low_pages: Vec::new(),
        }
    }

    fn qualifies(&self, level: Level) -> bool {
        match level {
            Level::High => !self.label_photos.is_empty() || !self.shop_match_pages.is_empty(),
            Level::Medium => self.medium_ids.len() >= 2,
            Level::Low => !self.low_ids.is_empty(),
        }
    }

    fn best(&self) -> Option<Level> {
        [Level::High, Level::Medium, Level::Low]
            .into_iter()
            .find(|l| self.qualifies(*l))
    }

    fn support(&self, level: Level) -> usize {
        match level {
            Level::High => self.label_photos.len() + usize::from(!self.shop_match_pages.is_empty()),
            Level::Medium => self.medium_ids.len(),
            Level::Low => self.low_ids.len(),
        }
    }

    fn sources(&self, level: Level) -> Vec<String> {
        match level {
            Level::High => self.shop_match_pages.clone(),
            Level::Medium => self.medium_pages.clone(),
            Level::Low => self.low_pages.clone(),
        }
    }
}

fn push_page(pages: &mut Vec<String>, page: &str) {
    if !pages.iter().any(|p| p == page) {
        pages.push(page.to_owned());
    }
}

fn group_for<'a>(
    groups: &'a mut Vec<Group>,
    definition: &Definition,
    stated: &Stated,
) -> &'a mut Group {
    let index = match groups
        .iter()
        .position(|g| same_statement(definition, &g.stated, stated))
    {
        Some(index) => index,
        None => {
            groups.push(Group::new(stated.clone()));
            groups.len() - 1
        }
    };
    &mut groups[index]
}

/// MC-A4 attribution decision over label items and listing items. Values are compared with
/// sameness; an unmapped statement is its own value and is never written.
pub fn decide_attribution(
    definition: &Definition,
    labels: &[LabelItem],
    listings: &[ListingItem],
) -> Decision {
    let mut groups: Vec<Group> = Vec::new();
    for label in labels {
        group_for(&mut groups, definition, &label.stated)
            .label_photos
            .insert(label.photo_id);
    }
    for item in listings {
        let verdict = match item.verdict {
            Some(v @ (Verdict::Match | Verdict::Similar)) => v,
            _ => continue,
        };
        let group = group_for(&mut groups, definition, &item.stated);
        group.low_ids.insert(item.site.identity.clone());
        push_page(&mut group.low_pages, &item.page);
        if verdict == Verdict::Match {
            match item.site.class {
                SiteClass::Shop => {
                    push_page(&mut group.shop_match_pages, &item.page);
                    group.medium_ids.insert(item.site.identity.clone());
                    push_page(&mut group.medium_pages, &item.page);
                }
                SiteClass::Trusted => {
                    group.medium_ids.insert(item.site.identity.clone());
                    push_page(&mut group.medium_pages, &item.page);
                }
                SiteClass::Untrusted => {}
            }
        }
    }
    // Only values with some level take part; others (a value only seen in a `different` page)
    // never reach here because such pages create no group.
    let mut ranked: Vec<(usize, Level, usize)> = groups
        .iter()
        .enumerate()
        .filter_map(|(i, g)| g.best().map(|l| (i, l, g.support(l))))
        .collect();
    let Some(top) = ranked.iter().map(|(_, l, _)| *l).max() else {
        return Decision::not_determined(Vec::new());
    };
    let at_top: Vec<usize> = ranked
        .iter()
        .filter(|(_, l, _)| *l == top)
        .map(|(i, _, _)| *i)
        .collect();
    let (winner, level) = if let [only] = at_top.as_slice() {
        (Some(*only), top)
    } else {
        let best = at_top
            .iter()
            .map(|i| groups[*i].support(top))
            .max()
            .unwrap_or(0);
        let leaders: Vec<usize> = at_top
            .iter()
            .copied()
            .filter(|i| groups[*i].support(top) == best)
            .collect();
        match leaders.as_slice() {
            [only] => (Some(*only), Level::Low),
            _ => (None, Level::Low),
        }
    };
    // named: by level, then support, then first appearance; the winner first when it cannot be
    // written (unmapped).
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0)));
    let others = |skip: Option<usize>| -> Vec<Stated> {
        ranked
            .iter()
            .filter(|(i, _, _)| Some(*i) != skip)
            .map(|(i, _, _)| groups[*i].stated.clone())
            .collect()
    };
    let Some(winner) = winner else {
        return Decision::not_determined(others(None));
    };
    match &groups[winner].stated {
        Stated::Mapped(value) => Decision {
            written: Some((value.clone(), level.confidence())),
            sources: groups[winner].sources(level),
            named: others(Some(winner)),
        },
        Stated::Unmapped(_) => Decision::not_determined(others(None)),
    }
}

/// MC-A4 measured fields: the attribution function over `match` listing items only, with a
/// `high` result lowered to `medium`. Never from our photos.
pub fn decide_measured(definition: &Definition, listings: &[ListingItem]) -> Decision {
    let matches: Vec<ListingItem> = listings
        .iter()
        .filter(|item| item.verdict == Some(Verdict::Match))
        .cloned()
        .collect();
    let mut decision = decide_attribution(definition, &[], &matches);
    if let Some((_, confidence)) = decision.written.as_mut()
        && *confidence == AiConfidence::High
    {
        *confidence = AiConfidence::Medium;
    }
    decision
}

/// MC-A4 visible fields: the photo read `v` against `w`, the attribution function over listing
/// items only (R-IA11).
pub fn decide_visible(
    definition: &Definition,
    photo_read: Option<&PhotoRead>,
    listings: &[ListingItem],
) -> Decision {
    let w = decide_attribution(definition, &[], listings);
    let Some(read) = photo_read else {
        return match w.written {
            Some((value, _)) => Decision {
                written: Some((value, AiConfidence::Low)),
                sources: w.sources,
                named: w.named,
            },
            None => Decision::not_determined(w.named),
        };
    };
    let agrees = w
        .value()
        .is_some_and(|value| same(definition, &read.value, value));
    let strong = matches!(
        w.confidence(),
        Some(AiConfidence::High | AiConfidence::Medium)
    );
    let confidence = if !read.sure {
        AiConfidence::Low
    } else if strong && agrees {
        AiConfidence::High
    } else if strong {
        AiConfidence::Low
    } else {
        AiConfidence::Medium
    };
    let mut named = w.named.clone();
    let sources = if agrees {
        w.sources.clone()
    } else {
        if let Some(value) = w.value() {
            named.insert(0, Stated::Mapped(value.clone()));
        }
        Vec::new()
    };
    Decision {
        written: Some((read.value.clone(), confidence)),
        sources,
        named,
    }
}

/// MC-A4 condition fields: from the stage 6 read over Condition photos only; listings are not an
/// input. With no Condition photo nothing is written.
pub fn decide_condition(photo_read: Option<&PhotoRead>, has_condition_photo: bool) -> Decision {
    match photo_read {
        Some(read) if has_condition_photo => Decision {
            written: Some((
                read.value.clone(),
                if read.sure {
                    AiConfidence::Medium
                } else {
                    AiConfidence::Low
                },
            )),
            sources: Vec::new(),
            named: Vec::new(),
        },
        _ => Decision::not_determined(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::sites::Site;
    use serde_json::json;

    fn def(core_type: CoreType, group: AttributeGroup, key: &str, validations: Json) -> Definition {
        Definition {
            integration: "test".into(),
            key: key.into(),
            label: "K".into(),
            core_type,
            choices: vec!["A".into(), "B".into()],
            validations,
            group,
            description: None,
            writable: true,
        }
    }

    fn maker() -> Definition {
        def(
            CoreType::Choice,
            AttributeGroup::Details,
            "custom.designer",
            json!({}),
        )
    }

    fn a() -> Stated {
        Stated::Mapped(json!("A"))
    }

    fn b() -> Stated {
        Stated::Mapped(json!("B"))
    }

    fn label(photo_id: i64, stated: Stated) -> LabelItem {
        LabelItem { photo_id, stated }
    }

    fn listing(
        class: SiteClass,
        identity: &str,
        page: &str,
        verdict: Verdict,
        stated: Stated,
    ) -> ListingItem {
        ListingItem {
            page: page.to_owned(),
            site: Site {
                class,
                identity: identity.to_owned(),
            },
            verdict: Some(verdict),
            stated,
        }
    }

    fn shop(page: &str, verdict: Verdict, stated: Stated) -> ListingItem {
        listing(SiteClass::Shop, "shop", page, verdict, stated)
    }

    fn trusted(entry: &str, verdict: Verdict, stated: Stated) -> ListingItem {
        listing(
            SiteClass::Trusted,
            entry,
            &format!("https://{entry}/p"),
            verdict,
            stated,
        )
    }

    fn untrusted(host: &str, verdict: Verdict, stated: Stated) -> ListingItem {
        listing(
            SiteClass::Untrusted,
            host,
            &format!("https://{host}/p"),
            verdict,
            stated,
        )
    }

    fn written(decision: &Decision) -> Option<(Json, AiConfidence)> {
        decision.written.clone()
    }

    use AiConfidence::{High, Low, Medium};
    use Verdict::{Different, Match, Similar};

    #[test]
    fn a05_c1a_label_high() {
        let d = decide_attribution(&maker(), &[label(1, a())], &[]);
        assert_eq!(written(&d), Some((json!("A"), High)));
    }

    #[test]
    fn a05_c1b_shop_match_high() {
        let d = decide_attribution(&maker(), &[], &[shop("https://beyo.example/p", Match, a())]);
        assert_eq!(written(&d), Some((json!("A"), High)));
    }

    #[test]
    fn a05_c1c_shop_similar_low() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[shop("https://beyo.example/p", Similar, a())],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
    }

    #[test]
    fn a05_c1d_two_trusted_sites_medium() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, a()),
                trusted("e2.example", Match, a()),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), Medium)));
    }

    #[test]
    fn a05_c1e_two_pages_one_entry_low() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                listing(
                    SiteClass::Trusted,
                    "e1.example",
                    "https://e1.example/1",
                    Match,
                    a(),
                ),
                listing(
                    SiteClass::Trusted,
                    "e1.example",
                    "https://e1.example/2",
                    Match,
                    a(),
                ),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
    }

    #[test]
    fn a05_c1f_trusted_and_untrusted_low() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, a()),
                untrusted("dealer.example", Match, a()),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
    }

    #[test]
    fn a05_c1g_label_beats_medium() {
        let d = decide_attribution(
            &maker(),
            &[label(1, a())],
            &[
                trusted("e1.example", Match, b()),
                trusted("e2.example", Match, b()),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), High)));
        assert_eq!(d.named, vec![b()]);
    }

    #[test]
    fn a05_c1h_two_labels_tie() {
        let d = decide_attribution(&maker(), &[label(1, a()), label(2, b())], &[]);
        assert_eq!(written(&d), None);
    }

    #[test]
    fn a05_c1i_greater_support_wins_at_low() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, a()),
                trusted("e2.example", Match, a()),
                trusted("e3.example", Match, b()),
                trusted("e4.example", Match, b()),
                trusted("e5.example", Match, b()),
            ],
        );
        assert_eq!(written(&d), Some((json!("B"), Low)));
        assert_eq!(d.named, vec![a()]);
    }

    #[test]
    fn a05_c1j_equal_support_not_determined() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, a()),
                trusted("e2.example", Match, a()),
                trusted("e3.example", Match, b()),
                trusted("e4.example", Match, b()),
            ],
        );
        assert_eq!(written(&d), None);
    }

    #[test]
    fn a05_c1k_similar_untrusted_majority_low() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                untrusted("one.example", Similar, a()),
                untrusted("two.example", Similar, a()),
                untrusted("three.example", Similar, b()),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
    }

    #[test]
    fn a05_c1l_unmapped_winner_not_determined() {
        let unmapped = Stated::Unmapped("unknown maker".to_owned());
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, unmapped.clone()),
                trusted("e2.example", Match, unmapped.clone()),
            ],
        );
        assert_eq!(written(&d), None);
        assert_eq!(d.named, vec![unmapped]);
    }

    #[test]
    fn a05_c1m_only_different_not_determined() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Different, a()),
                trusted("e2.example", Different, a()),
            ],
        );
        assert_eq!(written(&d), None);
    }

    #[test]
    fn a05_c1n_no_evidence_not_determined() {
        let d = decide_attribution(&maker(), &[], &[]);
        assert_eq!(written(&d), None);
    }

    fn length() -> Definition {
        def(
            CoreType::Dimension,
            AttributeGroup::Details,
            "width",
            json!({"units": ["CM", "MM"], "unit_names": {"cm": "CM", "mm": "MM"}}),
        )
    }

    fn cm120() -> Stated {
        Stated::Mapped(json!({"unit": "CM", "value": "120"}))
    }

    fn mm1200() -> Stated {
        Stated::Mapped(json!({"unit": "MM", "value": "1200"}))
    }

    #[test]
    fn a05_c2a_measured_same_value_two_sites_medium() {
        let d = decide_measured(
            &length(),
            &[
                trusted("e1.example", Match, cm120()),
                trusted("e2.example", Match, mm1200()),
            ],
        );
        assert_eq!(
            written(&d),
            Some((json!({"unit": "CM", "value": "120"}), Medium))
        );
    }

    #[test]
    fn a05_c2b_measured_high_capped() {
        let d = decide_measured(&length(), &[shop("https://beyo.example/p", Match, cm120())]);
        assert_eq!(
            written(&d),
            Some((json!({"unit": "CM", "value": "120"}), Medium))
        );
    }

    #[test]
    fn a05_c2c_measured_similar_not_counted() {
        let d = decide_measured(
            &length(),
            &[
                trusted("e1.example", Similar, cm120()),
                trusted("e2.example", Similar, cm120()),
            ],
        );
        assert_eq!(written(&d), None);
    }

    fn colour() -> Definition {
        def(
            CoreType::Choice,
            AttributeGroup::Details,
            "color",
            json!({}),
        )
    }

    fn read(stated: &str, sure: bool) -> PhotoRead {
        PhotoRead {
            value: json!(stated),
            sure,
        }
    }

    fn w_medium(stated: Stated) -> Vec<ListingItem> {
        vec![
            trusted("e1.example", Match, stated.clone()),
            trusted("e2.example", Match, stated),
        ]
    }

    #[test]
    fn a05_c3a_sure_read_agrees_with_medium_listing_high() {
        let d = decide_visible(&colour(), Some(&read("A", true)), &w_medium(a()));
        assert_eq!(written(&d), Some((json!("A"), High)));
    }

    #[test]
    fn a05_c3b_sure_read_disagrees_with_high_listing_low() {
        let d = decide_visible(
            &colour(),
            Some(&read("A", true)),
            &[shop("https://beyo.example/p", Match, b())],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
        assert!(d.named.contains(&b()));
    }

    #[test]
    fn a05_c3c_sure_read_with_low_listing_medium() {
        let d = decide_visible(
            &colour(),
            Some(&read("A", true)),
            &[untrusted("dealer.example", Similar, b())],
        );
        assert_eq!(written(&d), Some((json!("A"), Medium)));
    }

    #[test]
    fn a05_c3d_sure_read_no_listing_medium() {
        let d = decide_visible(&colour(), Some(&read("A", true)), &[]);
        assert_eq!(written(&d), Some((json!("A"), Medium)));
    }

    #[test]
    fn a05_c3e_unsure_read_low() {
        let d = decide_visible(
            &colour(),
            Some(&read("A", false)),
            &[shop("https://beyo.example/p", Match, a())],
        );
        assert_eq!(written(&d), Some((json!("A"), Low)));
    }

    #[test]
    fn a05_c3f_no_read_listing_low() {
        let d = decide_visible(&colour(), None, &w_medium(b()));
        assert_eq!(written(&d), Some((json!("B"), Low)));
    }

    #[test]
    fn a05_c3g_nothing_not_determined() {
        let d = decide_visible(&colour(), None, &[]);
        assert_eq!(written(&d), None);
    }

    #[test]
    fn a05_c4a_no_condition_photo_not_determined() {
        let d = decide_condition(Some(&read("Minor", true)), false);
        assert_eq!(written(&d), None);
    }

    #[test]
    fn a05_c4b_sure_read_medium() {
        let d = decide_condition(Some(&read("Minor", true)), true);
        assert_eq!(written(&d), Some((json!("Minor"), Medium)));
    }

    #[test]
    fn a05_c4c_unsure_read_low() {
        let d = decide_condition(Some(&read("Minor", false)), true);
        assert_eq!(written(&d), Some((json!("Minor"), Low)));
    }

    #[test]
    fn a05_c4d_none_for_unshown_part_medium() {
        let d = decide_condition(Some(&read("None", true)), true);
        assert_eq!(written(&d), Some((json!("None"), Medium)));
    }

    #[test]
    fn a05_c5a_field_class_first_match() {
        let keys = vec!["custom.wear".to_owned(), "custom.width".to_owned()];
        let wear = def(
            CoreType::Choice,
            AttributeGroup::Condition,
            "custom.wear",
            json!({}),
        );
        let width_attr = def(
            CoreType::Dimension,
            AttributeGroup::Details,
            "custom.width",
            json!({}),
        );
        let width = def(
            CoreType::Dimension,
            AttributeGroup::Details,
            "custom.depth",
            json!({}),
        );
        let colour = def(
            CoreType::Choice,
            AttributeGroup::Details,
            "custom.color",
            json!({}),
        );
        assert_eq!(field_class(&wear, false, &keys), FieldClass::Condition);
        assert_eq!(
            field_class(&width_attr, false, &keys),
            FieldClass::Attribution
        );
        assert_eq!(field_class(&width, false, &keys), FieldClass::Measured);
        assert_eq!(field_class(&colour, false, &keys), FieldClass::Visible);
        assert_eq!(field_class(&colour, true, &keys), FieldClass::NeverWritten);
    }

    #[test]
    fn a05_c6a_sources_in_given_order() {
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e2.example", Match, a()),
                trusted("e1.example", Match, a()),
            ],
        );
        assert_eq!(
            d.sources,
            vec![
                "https://e2.example/p".to_owned(),
                "https://e1.example/p".to_owned()
            ]
        );
    }

    #[test]
    fn a05_c6b_sources_label_only_and_majority() {
        let d = decide_attribution(&maker(), &[label(1, a())], &[]);
        assert!(d.sources.is_empty());
        let d = decide_attribution(
            &maker(),
            &[],
            &[
                trusted("e1.example", Match, a()),
                trusted("e2.example", Match, a()),
                trusted("e3.example", Match, b()),
                trusted("e4.example", Match, b()),
                trusted("e5.example", Match, b()),
            ],
        );
        assert_eq!(
            d.sources,
            vec![
                "https://e3.example/p".to_owned(),
                "https://e4.example/p".to_owned(),
                "https://e5.example/p".to_owned()
            ]
        );
    }

    #[test]
    fn a05_c6c_sources_at_high_are_the_shop_page() {
        let d = decide_attribution(
            &maker(),
            &[label(1, a())],
            &[
                shop("https://beyo.example/p", Match, a()),
                trusted("e1.example", Match, a()),
            ],
        );
        assert_eq!(written(&d), Some((json!("A"), High)));
        assert_eq!(d.sources, vec!["https://beyo.example/p".to_owned()]);
    }
}
