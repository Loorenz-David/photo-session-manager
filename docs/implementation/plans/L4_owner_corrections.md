---
phase: L4_owner_corrections
title: Price labels, the owner's corrections of 2026-11-24 — the check shows one easy-to-read card per product (picture, SKU, one-line title, price and outcome, warnings as pills) with the buttons always in view; Generate PDF saves every file and opens it in Preview at once, with no Download step; and products from the other Mac count as Published in the sessions list
difficulty: medium
projection: no
depends_on: [L3_generate_and_download]
blocked_on: none
---
# L4 — Price labels: the owner's corrections (09C §14)

**Tracker:** IMPLEMENTED (2026-11-25, opus, implement r1: U1 9cda2589, U2 7600a70b, U3 5d25b5f3; 51 rows tested; closing gate green — suite 1464 + crate tests, clippy, fmt, typecheck, npm test 948)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09C` **§14** (owner corrections 2026-11-24, changelog A34; §14.3 also amends 10H §6), by the rules **MC-L2.15–.17** (`docs/implementation/master_plan_labels.md` §5A) and **MC-H3.7A** (`docs/implementation/master_plan_archive.md` §5 MC-H3.7). (U1) §14.3: a shop-only item counts and filters as Published. (U2) §14.1: the check dialog's layout and its product cards, with the two things the cards need from the backend (the notes' kind and address, a picture per product). (U3) §14.2: Generate saves and opens every PDF at once; the dialog closes with a toast when everything was generated and stays on the problems otherwise. **Not here:** any change to how a run is made (keys, chunks, retries: L3 stands, M-L2, M-L3); a product from the other Mac **after its photos were brought back** (owner card 2, master plan §2A: it stays as today); live updating of a card's picture while the dialog is open; anything of 09C §11.

## How to work this plan (flow.md §10, master_plan_labels.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `L4·C<n>(<row>)`; tests named after their row (`l4_c1a_<slug>`). Open details: the smallest option consistent with 09C §14, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_labels/L4_owner_corrections.md`. **Never call Pricing, Shopify or a CDN, never launch the built app, never open Preview and never write into the real Documents folder from a test** (master plan §3: every `TestApp` uses `FakeLabelFiles`, `FakePricing`, `FakeCommerce` and its own temporary home). No company data in a test, fixture, log, handoff or commit (master plan §3: neutral values such as `TEST-1802`, "Test table", `https://shop.example/…`). `docs/implementation/STATUS.md` is never edited.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Products from the other Mac count as Published | 2 | 13 | **yes** |
| U2 | The check dialog: layout and product cards | 5 | 26 | **yes** |
| U3 | Generate goes straight to Preview | 2 | 12 | **yes** |
| **Total** | | **9** | **51** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · book `09C` **§14** (every sentence), §6 (what §14 amends), §9 (M-L2, M-L3, M-L4, M-L5) · book `10H` §3, §4, §6 · `docs/implementation/master_plan_labels.md` §3 (Rule 9, Rule 17, Secrets), §4 (Vocabulary, Words, Frontend, Tests), §5 MC-L1.3, MC-L1.4, MC-L2 (.1, .9, .10, .11), **§5A (MC-L2.15–.17)**, §2A card 2 · `docs/implementation/master_plan_archive.md` §5 **MC-H3.7** (with its L4 sentence, MC-H3.7A) · the L3 plan (`plans/L3_generate_and_download.md`: U3, U4 and their "Decided at projection r0" points D11, D15, D16, D17, which stand) · `docs/guide/03_rules_and_principles.md` (H1, H7).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A product from the other Mac counts as Published from its first open** (when the app has read its Shopify pictures). In the moments before that read, and for a shop product that has no picture at all, it stays under Open as today.
- **After its photos are brought back it is under Open, as today** (owner card 2 asks whether that should change).
- **The picture on a card is the one already on this Mac:** the session's row picture, else the Labels list's picture. A picture that is still downloading shows the neutral placeholder; the card does not update while the dialog is open.
- **"QR link" appears only when Pricing's note names an address.** The address is read out of Pricing's sentence; if Pricing rewords it, the card shows "Not published yet" without the button.
- **A product Pricing did not find keeps the dialog open after Generate** and counts in the title ("2 of 3 labels generated"), although it was already shown as "Not found in Pricing" before.
- **The toast says "Label generated · opened in Preview" whenever exactly one label was made,** also from a batch.
- **A file that was saved but that Preview could not open is not offered again** (the dialog says where it is), so no second copy is saved.
- **The result still lists what happened in numbers** (marked Printed, reprinted, stay Not printed, failed) above the problem cards.
- **The dialog cannot be closed while the files are being saved and opened** (a few seconds after Generate).

## Unit 1 — Products from the other Mac count as Published

**Unit id:** `L4·U1` · **Rows:** L4·C1–C2 · **Changes app files:** yes

### Goal
`SessionsService::list` (and so `page`) gives a `Remote(ShopOnly)` row the status `Published` unless a publish of it shows through, so the Open / Published counts, the two filters, the paged list and every caller of the list agree. No frontend file changes: the screen reads `row.status` and `counts` from the two commands.

### Read first
`master_plan_archive.md` §5 MC-H3.7 and MC-H3.7A · `crates/core/src/services/sessions.rs` (`list` :453-591: the counts and the filter at :497-511, the label rule at :512-529; `page` :599-657 calls `list`; `status_input` :661; `ready_session_ids` :681, **unchanged**) · `crates/core/src/mechanisms/status.rs` (`derive_status`, `row_label`: **unchanged**) · `crates/core/src/services/remote.rs` (`RowPictures::remote_row` :202, `RemoteRow.state`) · `crates/core/src/mechanisms/remote.rs` (`remote_state` :169) · `apps/desktop/src-tauri/src/commands/sessions.rs` (`sessions_page` :461-500 and `sessions_list` :502-512 both attach `.with_remote`) · `crates/testkit/tests/suite/h4_details_only_publish.rs` (helpers `app`, `shop_only` :81 (over `adopted` of `h3_bring_back.rs` :194), `archived_in_step` :494, `row`, `counts`, `set_text`, `confirm`, `drain`; rows H4·C9 :447-548) · `apps/desktop/src-tauri/src/lib.rs` (`h2_tests` :2150: how a test calls the two commands' bodies).

### Files
Edited: `crates/core/src/services/sessions.rs`, `crates/testkit/tests/suite/h4_details_only_publish.rs` (the L4·C1 tests live beside H4·C9, whose helpers they use; `h4_c9a_a_shop_only_row_counts_its_record` is **replaced** by `l4_c1a_…`), `apps/desktop/src-tauri/src/lib.rs` (`mod l4_tests`), `docs/guide/02_architecture.md` (the "Rows (MC-H3.7)" sentence, :346).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. In `list`, per row, after `input` and `remote_row` are known: compute the label first (the existing rule, unchanged), then the status: **when `remote_row` is `Some` with `state == Remote(ShopOnly)` and the label is `RowLabel::Archived` → `SessionStatus::Published`; otherwise `derive_status(input)` as today** (MC-H3.7A). The counts (`open`, `published`, `all`), the Open / Published filter and `SessionRow.status` all use that one value. Archived rows, `Unknown` rows, local rows and a list built without `with_remote` are untouched.
2. Tests for the rows; replace `h4_c9a`.
3. Guide: `02_architecture.md` "Rows (MC-H3.7)": one sentence for the shop-only status.

### Criteria (2 criteria, 13 rows)
Boundary for C1: `SessionsService` with `.with_remote(a.remote())` on the app of `h4_details_only_publish.rs` (a fake shop; outcome = the returned rows and counts). "SO" = `shop_only(&a, <sku>, &[3 gallery ids], &[1 condition id])` (adopted, first-open read done, never published from this Mac). "In step S" = its details were edited and its details-only publish confirmed, then put in step S through the store, as `archived_in_step` does for an archived item. For C2: the bodies of the two commands in the desktop crate's `l4_tests`, on a `TestApp` with the fake shop.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L4·C1(a) | `list(All, None)` | SO | its row: `main_count` 3, `condition_count` 1, `unreviewed_count` 0, status `Published`, label `Archived` | 09C §14.3; MC-H3.7A |
| L4·C1(b) | `list(All, None).counts` before and after SO is adopted and read | — | `published` is one higher, `open` is unchanged, `all` is one higher | 09C §14.3 ("counts … as Published") |
| L4·C1(c) | `list(Open, None)`; `list(Published, None)` | SO | no row with SO's session id; exactly one row with it | 09C §14.3 ("filters as Published") |
| L4·C1(d) | `page(Published, None, None, None, 1)`; `page(Open, None, None, None, 1)` | SO | the first holds SO's row with status `Published`, the second does not; both answers' `counts` equal `list(All, None).counts` | 09C §14.3; MC-H2.1 |
| L4·C1(e) | `list(All, None)`; `list(Open, None)` | SO in step `Update` | status `Publishing`, label `Publishing(75)`; the Open list holds the row | 09C §14.3 ("a publish of it running"); MC-H3.7A |
| L4·C1(f) | `list(All, None)` | SO in step `Held` | status `Publishing`, label `WaitingForYou(0)` | 09C §14.3 ("held") |
| L4·C1(g) | `list(All, None)`; `list(Published, None)` | SO in step `Failed` (failed step `Update`) | status `Failed`, label `PublishFailed`; the Published list does not hold the row | 09C §14.3 ("failed") |
| L4·C1(h) | `list(All, None)`; `list(Published, None)` | SO in step `Cancelled` | status `Ready`, label `PublishCancelled`; the Published list does not hold the row | 09C §14.3 ("cancelled") |
| L4·C1(i) | `list(All, None)` | SO after its details-only publish ran to `Done` (`confirm`, `drain`) | status `Published`, label `Archived` | 09C §14.3; MC-H3.7A |
| L4·C1(j) | `list(All, None)`; `list(Open, None)` | `adopted(…)` whose first-open read has not run (state `Unknown`) | status `InProgress`, label `InProgress(0)`; the Open list holds the row (unchanged by L4) | MC-H3.7A (only `Remote(ShopOnly)`) |
| L4·C1(k) | `list(All, None)` | an adopted product whose shop record has no picture, read (state `Local`) | status `InProgress`, label `InProgress(0)` (unchanged by L4) | MC-H3.7A |
| L4·C2(a) | `sessions_page` (filter `published`, page 1) | SO | a row for SO with `status: "published"` and `label: {kind: "archived"}`; `counts.published` is one higher than before SO existed and `counts.open` is unchanged | 09C §14.3; MC-H2.2 |
| L4·C2(b) | `sessions_list` (filter `all`) | SO | the same row status and exactly the `counts` of (a) | 09C §14.3 (the header, the pickers and the workspace panel read this list) |

### Delegated choices
Whether the status rule is written inline in `list` or as a small private function beside `status_input` · the SKUs and media ids of the fixtures.

### Notes
- Hazard: the label must be computed **before** the status (the rule reads it), and from the same `input`; today the status comes first (:497) and the label after the filter (:512). Rows cut by the filter never reach the label code today.
- Hazard: do not change `derive_status`, `row_label` or `ready_session_ids`: a shop-only item never enters the "ready" set (H4 projection r0, L8), and archived rows keep H4·C9(b)–(g) exactly.
- What follows on screen without a frontend change: the row is no longer tickable for Review & publish (`SessionRow.tsx:20` ticks `ready` / `failed` rows) and leaves the workspace's sessions panel (`SessionsPanel.tsx:15` lists rows that are not `published`). Both are what "counts as Published" means; no test of those two files changes.
- `h4_c9a` is the only earlier test this unit replaces; H4·C9(b)–(g) must pass unchanged.
- A brought-back shop-only item is `Local` (it has photo rows), so this rule no longer applies to it: it is under Open as today (owner card 2). No row asserts it.

### Unit test commands
`cargo test -p psm-testkit --test suite h4_details_only_publish::` · `cargo test -p psm-testkit --test suite h2_` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-testkit -p psm-desktop --all-targets -- -D warnings`

## Unit 2 — The check dialog: layout and product cards

**Unit id:** `L4·U2` · **Rows:** L4·C3–C7 · **Changes app files:** yes

### Goal
The check's notes keep their kind and their address (adapter → core → command); the check names a picture per product from what is already on this Mac; the dialog has a fixed height with a pinned footer and a scrolling list of product cards (09C §14.1).

### Read first
Book `09C` §14.1 · master plan §5A **MC-L2.15, MC-L2.16**, §5 MC-L1.3, MC-L1.4 · `crates/core/src/domain/label.rs` (`PlanItem` :147-149, `LabelCheck`) · `crates/integrations/src/conversations/beyo_pricing/{labels.rs (`plan_item` :148-172; the inline test `l1_c7c_plan_reprint_and_nullable_template_and_title` :1249), label_types.rs (`IntegrationPlanIssue` :173-179)}` · `crates/core/src/services/labels.rs` (`seen_skus` :112, :150, :349-353, :411-415, :558; `match_items` :384; `list` :401-477 with the picture rule :444-451; `select_all` :507) · `crates/core/src/services/label_runs.rs` (`check` :130-203) · `crates/core/src/services/remote.rs` (`search_picture` :442: answers the cached file or queues the download, never downloads) · `apps/desktop/src-tauri/src/commands/labels.rs` (the check item DTO :485-486, :619-620) · `crates/testkit/src/fake_pricing.rs` (`LabelSpec` :62, the plan item built at :618 with `notes: vec![]`) · `crates/testkit/tests/suite/l2_labels_list.rs` (helpers `linked`, `product`, `search_file`, `run_pending`; rows L2·C3 :439-540) · `apps/desktop/src/features/labels/{LabelDialog.tsx, LabelCheckView.tsx, useLabelDialog.ts, labelWords.ts, LabelRow.tsx}` · `apps/desktop/src/features/sessions/HoverPreview.tsx` (`Thumb`) · `apps/desktop/src/components/ExternalLink.tsx` · `apps/desktop/src/styles/globals.css` (:253-254, :297-298 the row thumbnail; :519-553 the label dialog) · `apps/desktop/src/features/labels/__tests__/l3_label_check_dialog.test.tsx`.

**Rule 17, the one outside shape of this milestone.** A plan item's `notes[]` entry is `{code, message}` and nothing else (`openapi-integration.json` component `IntegrationPlanIssue`: both required, `additionalProperties: false`). `API.md` › `POST …/labels/plan` names the code `url_not_published` ("the QR code opens the webshop search until the product is published") and shows no example of its message. The message Pricing builds was read on 2026-11-25 in its source, `pricing-app/backend/app/domains/labels/plan.py:196-200`: `f"Not published to the Online Store yet; the QR code opens {url}."`. **There is no address field: the address is inside the sentence, followed by a full stop.** The fixtures of L4·C3 keep that sentence form with a neutral address. This is source-grounded, not package-grounded: the package does not promise the wording, which is why MC-L2.15 fails soft (no address → no "QR link").

### Files
New: `crates/testkit/tests/suite/l4_labels_check_cards.rs` (with its `mod` line in `main.rs`), `apps/desktop/src/features/labels/LabelCard.tsx`, `apps/desktop/src/features/labels/__tests__/l4_label_check_cards.test.tsx`. Edited: `crates/core/src/domain/label.rs`, `crates/core/src/services/{labels.rs, label_runs.rs}`, `crates/core/src/mechanisms/labels.rs` (only where a test builds a `PlanItem`), `crates/integrations/src/conversations/beyo_pricing/labels.rs`, `crates/testkit/src/fake_pricing.rs`, `crates/testkit/tests/suite/main.rs`, `apps/desktop/src-tauri/src/{lib.rs (`l4_tests`; `l3_c11a_labels_check_answers_the_groups` gains `picture_path: null` per item), commands/labels.rs}`, `apps/desktop/src/services/{bindings.ts (generated), labels.ts}`, `apps/desktop/src/features/labels/{LabelDialog.tsx, LabelCheckView.tsx, labelWords.ts}`, `apps/desktop/src/styles/globals.css`, `apps/desktop/src/features/labels/__tests__/l3_label_check_dialog.test.tsx` (see "Earlier tests"), `docs/guide/{02_architecture.md, 04_change_impact.md}`.

**Earlier tests this unit changes, and no others.** `l1_c7c_plan_reprint_and_nullable_template_and_title` (adapter, inline): its note assertion becomes the `PlanNote` of L4·C3(b). `l3_c11a_labels_check_answers_the_groups` (desktop): the expected JSON gains `picture_path: null` on each item (L3·C11(a) otherwise unchanged). `l3_c12b_one_section_per_template_with_a_line_per_product` (UI): **removed**, replaced by L4·C6(b), (c). In the other tests of `l3_label_check_dialog.test.tsx`, `l3_label_done_dialog.test.tsx` and `l3_label_wiring.test.tsx` only fixtures may change in this unit (`picture_path: null` on check items); every expectation stays.

### Core mechanisms (the only functions with unit tests)
None (the address rule is the adapter's and is proven at the adapter's boundary, L4·C3).

### Ordered tasks
1. Vocabulary (master plan §4, L4): `NoteKind {NotPublished, Other}`, `PlanNote {kind, message: String, link: Option<String>}`; `PlanItem.notes: Vec<PlanNote>`; `LabelCheck.pictures: HashMap<String, PathBuf>` (product id → a picture file that exists). No name of Pricing or Shopify in core (H7).
2. Adapter (MC-L2.15): `plan_item` maps each note: code `url_not_published` → `NotPublished`, any other code → `Other`; `message` as given; `link` only for `NotPublished`: the first whitespace-separated word of the message that starts with `https://` or `http://`, with trailing `.`, `,`, `;` and `)` removed; none → `None`.
3. Service (MC-L2.16): the remembered rows keep their picture address: `seen_skus` becomes one map product id → (SKU, `thumb_url`), filled by `list` and by `select_all` (every row of every page it walks), emptied where it is emptied today (`forget`). `check`, after the plan is merged and before the check is kept: `match_items()` once; for each item `match_product(product_id, sku, items)` → that item's latest session; when at least one matched, **one** `SessionsService::new(..).with_remote(..).list(ListFilter::All, None)` and each matched session's `cover_preview_path` (present only when its file exists, M-H4) is the picture; for an item without a session picture: the remembered `thumb_url`, when it starts with `https://`, `commerce_id_of(product_id)` is known and a commerce integration is present → `RemoteItems::search_picture(commerce_id, thumb_url)` (the cached file, or none while its download waits in 10H's queue). No picture → no entry. A failure of any of these reads leaves `pictures` empty and never fails the check.
4. Command: the check item DTO gains `notes: [{kind: 'not_published' | 'other', message, link: string | null}]` and `picture_path: string | null`; bindings regenerated (never hand-edited); `services/labels.ts` types follow.
5. `FakePricing`: `note_label(product_id, code, message)` (a plan item of that product carries the note; caller L4·C5(a)).
6. `labelWords.ts`: `NO_PRICE_PILL = 'No price'` (the existing `NO_PRICE`), `NOT_PUBLISHED_YET = 'Not published yet'`, `QR_LINK = 'QR link'`, `cantGenerate(message)` = `Can't generate: <message>`; `groupLines(group)` = `{name, detail}`: `name` = the template's name or `No template`; `detail` = `<product types joined by ", "> · <size>`, only the size when there are no product types, none without a template (replaces `groupHeading`).
7. `LabelCard` (09C §14.1, one per product): left a `Thumb` in a 1:1 square (`label-card-thumb`, the sessions-list thumbnail's width) with the SKU (else the product id) under it; right, line 1 the title on one line (ellipsis by CSS, `title` attribute = the full title); line 2 `formatPrice(price_minor)` and the outcome words; under the content a warnings row, rendered only when it has a pill, in this order: `No price` (amber) when `outcome` is `no_price` or `price_minor` is null · one red pill `Can't generate: <issue>` per issue · for each `not_published` note `Not published yet` (amber) and, when it has a `link`, the pill button `QR link` · each `other` note as a grey pill with its message. `QR link` toggles a line under the pills showing the address as an `ExternalLink` (pressing the pill opens nothing; clicking the address opens it in the browser). The card is a button-like element: clicking it selects it (the preview shows that product); the selected card carries `aria-current="true"`.
8. `LabelCheckView` / `LabelDialog` layout: the dialog content is a column of fixed height (`min(720px, calc(100vh - 4rem))`, never growing with the list; the exact value is the implementer's): the header; a body row with the **scrolling** list (`data-testid="label-check-scroll"`: the groups, their cards, the "Not found in Pricing" lines) and the preview pane beside it; the **footer** (`data-testid="label-check-footer"`: the summary sentences, a Generate failure's message, and the buttons), outside the scrolling element. A group's header is two lines (`groupLines`), each on one line with an ellipsis and its full text as `title`. `‹` `›` and the preview behave as in L3 (D15); `useLabelDialog` gains `show(productId)`.
9. Guide: `02_architecture.md` §Labels (the check carries notes with kind and address, and a picture per product from the row picture or the search cache; no new call), `04_change_impact.md` (row: the check item DTO → bindings, `services/labels.ts`, `LabelCard`).
10. Tests for the rows.

### Criteria (5 criteria, 26 rows)
Boundary for C3: `BeyoPricingAdapter::plan` over the recording transport (inline tests in `beyo_pricing/labels.rs`, built with `plan_with`). For C4: `App.labels` on `TestApp::with_pricing()`, connected and `Allowed`, with the fake shop (outcome = `check(..).pictures`, the recorded label requests, the fake shop's reads and `mutation_ops()`). For C5: the body of `labels_check` in the desktop crate's `l4_tests`. For C6–C7: `LabelDialog` rendered in UI tests with `@/services/labels` mocked and `openUrl` of `@tauri-apps/plugin-opener` mocked (outcome = what is rendered and the exact lists of calls). `ADDR` = `https://shop.example/search?q=TEST-1802`; `SENT` = `Not published to the Online Store yet; the QR code opens https://shop.example/search?q=TEST-1802.`; `<thin>` in a price is U+2009.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L4·C3(a) | `plan` | an item with `notes: [{"code": "url_not_published", "message": SENT}]` | that item's `notes` = `[PlanNote {kind: NotPublished, message: SENT, link: Some(ADDR)}]` (no trailing full stop in the link) | MC-L2.15; 09C §14.1 ("the note's address"); Rule 17 |
| L4·C3(b) | `plan` | `notes: [{"code": "url_not_published", "message": "M"}]` | `[PlanNote {kind: NotPublished, message: "M", link: None}]` | MC-L2.15 (fails soft) |
| L4·C3(c) | `plan` | `notes: [{"code": "something_else", "message": "See https://shop.example/a."}]` | `[PlanNote {kind: Other, message: "See https://shop.example/a.", link: None}]` | MC-L2.15; 09C §14.1 ("any other note") |
| L4·C3(d) | `plan` | code `url_not_published` with message `Opens http://shop.example/a, soon.`; with message `Opens ftp://shop.example/a.` | `link: Some("http://shop.example/a")`; `link: None` | MC-L2.15 |
| L4·C4(a) | `check(["1"])` | P1 linked to a session on this Mac whose first main photo's preview file exists | `pictures` = `{"1": <that session's row picture, as `sessions().with_remote(..).list(All, None)` answers its `cover_preview_path`>}`; the recorded label requests are one plan and one templates request; the fake shop saw no read | MC-L2.16; 09C §14.1 ("the one the product's sessions-list row shows") |
| L4·C4(b) | `list(All, None, None)`, the background queue run (`run_pending`), then `check(["2"])` | P2 has no session here and a `thumb_url` starting `https://` | `pictures` = `{"2": <the search cache file of P2's shop id>}` | MC-L2.16; 09C §14.1 ("else Pricing's `thumbUrl` known from the Labels list") |
| L4·C4(c) | `list(All, None, None)`, then `check(["2"])` **without** running the queue; then the queue run and `check(["2"])` again | same | first `pictures` is empty and the search cache file does not exist after the call; then `pictures` holds the file | MC-L2.16 ("never downloads"); MC-L1.4 |
| L4·C4(d) | `check(["2"])` on a fresh app (no `list` before) | same | `pictures` is empty; the fake shop saw no read and its download queue stays empty after a queue run | MC-L2.16; 09C §14.1 ("no extra Pricing or Shopify call per card") |
| L4·C4(e) | `select_all(All, None)`, `check(["2"])`, the queue run, `check(["2"])` | same | the second check's `pictures` holds P2's search cache file | MC-L2.16 (Select all N remembers too) |
| L4·C4(f) | `list(All, None, None)`, the queue run, Test connection (`connected()`), `check(["2"])` on a fake with the cache file removed | same | `pictures` is empty and no download is queued (the remembered addresses were forgotten with the SKUs) | MC-L2.16; ACC.2 |
| L4·C4(g) | after (a)–(f) | — | no request line contains `/labels/runs`; the fake shop's `mutation_ops()` is empty | M-L2; M-L5 |
| L4·C5(a) | `labels_check(["1"])` | `note_label("1", "url_not_published", SENT)`; P1 has no session and was never listed | the item's JSON has `notes: [{"kind": "not_published", "message": SENT, "link": ADDR}]` and `picture_path: null` | MC-L2.15; Commands |
| L4·C5(b) | `labels_check(["1"])` | P1 as in L4·C4(a), no note | `notes: []` and `picture_path` = that picture's path as text | MC-L2.16; Commands |
| L4·C6(a) | `LabelDialog(["1", "2"])` with a check answer | — | the element `label-check-scroll` holds every product card and no button named `Generate PDF` or `Cancel`; the element `label-check-footer` holds the summary sentence(s), `Generate PDF` and `Cancel`, and is not inside `label-check-scroll`; the preview is in neither | 09C §14.1 ("the footer … always visible; only the list … scrolls") |
| L4·C6(b) | same | groups: `Label 70×100` (product types `Tables`, `Chairs`, 70 × 100); `Small tag` (none, 50 × 30); a group without a template | per group two separate lines: `Label 70×100` and `Tables, Chairs · 70 × 100 mm`; `Small tag` and `50 × 30 mm`; `No template` and no second line; each line's `title` attribute equals its text | 09C §14.1 ("Template group header") |
| L4·C6(c) | same | four items, one per outcome: `mark_printed` (SKU `TEST-1`, title `Test table`, 890000, `picture_path: "/d/previews/p/1.jpg"`), `reprint`, `no_price` (`price_minor: null`), `fail` | card 1: an image of `previewUrl("/d/previews/p/1.jpg")` inside an element with the class `label-card-thumb`, `TEST-1` in the same side column after it, the title `Test table` in an element whose `title` attribute is `Test table`, then `8<thin>900 kr` and `Will be marked Printed`; the others read `Reprint · stays Printed`, `No price` with `No price · stays Not printed`, `Can't generate` | 09C §14.1 ("One card per product") |
| L4·C6(d) | same | an item with `picture_path: null`; an item with `sku: null` (product id `77`) | no image in its card (the thumbnail's empty tile); `77` under the picture | 09C §14.1 ("else the neutral placeholder") |
| L4·C6(e) | a batch of three | click the third card | the first card has `aria-current="true"` at first; after the click only the third has it and `labelsPreview` was asked once for the third product; `‹` is then enabled and `›` disabled | 09C §14.1 ("clicking a card shows its preview") |
| L4·C7(a) | a card | an item with a price, outcome `mark_printed`, no issue, no note | the card has no warnings row (no element with the class `label-card-warnings`) | 09C §14.1 ("only when there is something to say") |
| L4·C7(b) | a card | outcome `no_price`, `price_minor: null` | one pill `No price` with `data-tone="amber"` | 09C §14.1 |
| L4·C7(c) | a card | outcome `fail`, `issues: ["Barcode: no barcode.", "No template applies."]` | two pills with `data-tone="red"`: `Can't generate: Barcode: no barcode.` and `Can't generate: No template applies.` | 09C §14.1; 09C §10 (the message as given) |
| L4·C7(d) | a card | `notes: [{kind: "not_published", message: SENT, link: ADDR}]`; press `QR link`; click the address; press `QR link` again | a pill `Not published yet` (`data-tone="amber"`) and a button `QR link`; `SENT` and `ADDR` are not on screen at first; after the press `ADDR` is shown under the pills and `openUrl` was not called; the click calls `openUrl(ADDR)` exactly once; the second press hides `ADDR` | 09C §14.1 ("QR link … never opens anything by itself") |
| L4·C7(e) | a card | `notes: [{kind: "not_published", message: "M", link: null}]` | `Not published yet`; no `QR link` | MC-L2.15 |
| L4·C7(f) | a card | `notes: [{kind: "other", message: "Something to know."}]` | one pill `Something to know.` with `data-tone="grey"` | 09C §14.1 ("any other note as a grey pill") |
| L4·C7(g) | a card | outcome `fail` with one issue, `price_minor: null`, a `not_published` note with a link | the pills in this order: `No price`, `Can't generate: …`, `Not published yet`, `QR link` | 09C §14.1 (the list's order) |
| L4·C7(h) | a batch with `unresolved: [{product_id: "99", sku: null, …}]` | — | `99 · Not found in Pricing` inside `label-check-scroll`, outside every group, with no card | 09C §6.1; §14.1 |

### Delegated choices
The dialog's exact height and the card's spacing and tokens (design handoff tokens; "a simple well made card") · the pill element (the existing `pill` class with a tone) · whether `LabelCard` is used as is for U3's problem cards (it is meant to be) · how the remembered map is named · whether the card listens for a late picture (it need not).

### Notes
- Hazard: `SessionsService::page(.., Some(&ids), 1)` (what `list` of the Labels box uses, MC-L1.3) is cut to 25 rows; a check can hold hundreds of products. Use `list(ListFilter::All, None)` once and pick the matched sessions, as task 3 says. `list` never requests a cover download; a remote item's cover that is not on disk yet simply gives no picture.
- Hazard: jsdom lays nothing out, so the fixed height, the scrolling and the one-line ellipsis cannot be asserted; L4·C6(a) asserts the structure that makes them possible and Review mode reads the CSS.
- Hazard: the picture path of a card is inside the data folder (`previews/…`) and goes through `previewUrl`; the address of "QR link" is never loaded, only opened through `openExternal` (http / https only).
- Hazard: `picture_path` is a display fact of this Mac, not label state; nothing of it is stored (M-L1 holds: no migration, no column).
- Hazard (H7): core says `NotPublished`; the code `url_not_published` and the sentence's form are known only to the adapter.
- `1 products will be marked Printed` stays Pricing's wording (L3 hazard).

### Unit test commands
`cargo test -p psm-integrations --lib conversations::beyo_pricing::labels` · `cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-testkit --test suite l4_labels_check_cards::` · `cargo test -p psm-testkit --test suite l3_labels_check::` · `cargo test -p psm-testkit --test suite l2_labels_list::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels` · `npm run typecheck` · `cargo clippy -p psm-core -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings`

## Unit 3 — Generate goes straight to Preview

**Unit id:** `L4·U3` · **Rows:** L4·C8–C9 · **Changes app files:** yes (frontend only)

### Goal
After a run the dialog itself saves and opens every file (the existing `labels_download`, once per file), closes with a toast when everything was generated, and otherwise stays on the result with the problems as cards. No Download PDF step, no Done button. **No Rust file changes in this unit.**

### Read first
Book `09C` §14.2, §6.2, §6.5, §10 · master plan §5A **MC-L2.17**, §5 MC-L2.9–.11 · `apps/desktop/src/features/labels/{useLabelDialog.ts (`generate` :125, `leave` :141, `download` :158), LabelDialog.tsx, LabelDoneView.tsx, labelWords.ts (`fullyGenerated`, `doneTitle`, `resultLines`), LabelsBox.tsx (`batchClosed` around :200)}` · U2's `LabelCard` · `apps/desktop/src/features/labels/__tests__/{l3_label_done_dialog.test.tsx, l3_label_check_dialog.test.tsx (`l3_c14d` :314), l3_label_wiring.test.tsx (`l3_c17e` :235)}` · `apps/desktop/src/features/sessions/DeleteSession.tsx` (:4, :20: how a toast is raised with `sonner`).

### Files
New: `apps/desktop/src/features/labels/__tests__/l4_label_result.test.tsx`. **Deleted:** `apps/desktop/src/features/labels/__tests__/l3_label_done_dialog.test.tsx` (its 10 tests, rows L3·C15–C16, state the done / download dialog that 09C §14.2 supersedes; what still holds of them is restated in L4·C8–C9). Edited: `apps/desktop/src/features/labels/{useLabelDialog.ts, LabelDialog.tsx, LabelDoneView.tsx, labelWords.ts}`, `apps/desktop/src/styles/globals.css`, `apps/desktop/src/features/labels/__tests__/{l3_label_check_dialog.test.tsx, l3_label_wiring.test.tsx}`, `docs/guide/02_architecture.md`.

**Earlier tests this unit changes, and no others.** Deleted with their file: `l3_c15a`, `l3_c15b`, `l3_c15c`, `l3_c15d`, `l3_c15e`, `l3_c16a`, `l3_c16b`, `l3_c16c`, `l3_c16d`, `l3_c16e`. `l3_c14d_leaving_after_a_failed_generate_shows_what_was_generated`: **removed**, replaced by L4·C9(d). `l3_c17e_after_a_run_the_failed_products_stay_selected_and_the_page_is_asked_again`: its expectation stays (the selection is exactly the failed product, one refresh); only its mocks may change (`labelsDownload` answered for the result's files, the dialog left with `Close`). Every other L3 test keeps its expectation.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `labelWords.ts`: `doneTitle(result, single, notFound)` with `notFound` = the check's `unresolved` count: one product → `Label generated` when fully generated, `notFound` is 0 and `rendered > 0`, else `Label not generated`; a batch → `<total> labels generated` when fully generated and `notFound` is 0, else `<rendered> of <total + notFound> labels generated`. `generatedToast(result)` = `Label generated · opened in Preview` when `rendered` is 1, else `<rendered> labels generated · opened in Preview`, plus ` · <no_price> stay Not printed (no price)` when `no_price > 0`. `DOWNLOAD_PDF` and `DONE` are removed.
2. `useLabelDialog` (MC-L2.17): whenever the dialog obtains a result (from `labelsGenerate` or from `labelsPartial`), it stays in a busy state that cannot be closed and calls `labelsDownload({check_id, artifact_id})` for each of the result's files, in order, one after the other, **each file at most once automatically per dialog** (a ref of the ids asked). When all have settled: **everything was generated** ⇔ `fullyGenerated(result)` ∧ the check's `unresolved` is empty ∧ every file answered `opened: true` → `toast.success(generatedToast(result))` and `close(result)`; otherwise the result view. `retry(file)` calls `labelsDownload` again for a file whose save failed with a retryable code (`canRetryDownload`, unchanged); a later success updates that file's line and the dialog stays open.
3. `LabelDoneView` (the result view): the result lines (`resultLines`, unchanged) · one `LabelCard` per failure (picture, SKU, title and price from the check's item of that product id when there is one, else the failure's own SKU and title; outcome words `Can't generate`; one red pill `Can't generate: <the failure's message>`) · the check's unresolved entries as `<SKU or product id> · Not found in Pricing` · `Failed products stay selected so you can fix and retry.` for a batch that is not fully generated (unchanged rule) · one line per file: `Saved and opened in Preview: <path>`, `Saved, but Preview couldn't open it: <path>` (no button), or the failure's message with `Try again` when retryable · `Print at 100 % (not Fit to page).` when at least one file was saved · the single button `Close`. The view scrolls inside the dialog's fixed height with `Close` pinned, as the check's footer is.
4. `LabelDialog`: the title by the new `doneTitle`; the busy line while saving reads `Opening in Preview…`; Escape, a click outside and the buttons do nothing while generating **or saving**.
5. Guide: `02_architecture.md` §Labels frontend line (Generate → save and open each file → toast or result view; `labels_download` is called by the dialog, never by a button).
6. Tests for the rows.

### Criteria (2 criteria, 12 rows)
Boundary: `LabelDialog` (and for L4·C8(d) the Labels box) rendered in UI tests with `@/services/labels` and `sonner`'s `toast` mocked; outcome = what is rendered, the exact lists of `labelsGenerate`, `labelsPartial`, `labelsDownload` and `toast.success` calls, and what `onClose` received. "A check answer" has the shape of L3·C11(a) with U2's fields. `SAVED(p)` = `labelsDownload` answering `{path: p, opened: true}`. In every row: no element named `Download PDF` or `Done` exists at any time.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L4·C8(a) | `LabelDialog(["1"])`, press `Generate PDF` | result `{total: 1, rendered: 1, marked_printed: 1, artifacts: [a1]}`; `SAVED` | with no further press: exactly one `labelsDownload({check_id, artifact_id: "a1"})`; `toast.success` called once with `Label generated · opened in Preview`; `onClose` received the result; the dialog is gone | 09C §14.2; MC-L2.17; §12.3 |
| L4·C8(b) | `LabelDialog(["1", "2", "3"])`, press `Generate PDF` | result `{total: 3, rendered: 3, marked_printed: 2, no_price: 1, artifacts: [a1 (70 × 100), a2 (50 × 30)]}`; `SAVED` for both | exactly two `labelsDownload` calls, `a1` then `a2`; `toast.success` once with `3 labels generated · opened in Preview · 1 stay Not printed (no price)`; the dialog is gone | 09C §14.2 ("every file the run made"); §12.5 |
| L4·C8(c) | `LabelDialog(["1"])`, press `Generate PDF` | the result arrives; `labelsDownload` stays pending; Escape; a click outside | the dialog is still open with the busy line `Opening in Preview…`; no enabled button; `toast.success` not called; exactly one `labelsDownload` | MC-L2.17 (not closable while saving) |
| L4·C8(d) | the Labels box with rows "3" and "1" ticked, `Generate labels for 2`, `Generate PDF` | a fully generated result with one file; `SAVED` | the dialog closes by itself; the selection is empty; exactly one `labelsList` refresh for the page shown; one `toast.success` | 09C §14.2; §6.5; MC-L1.9 |
| L4·C9(a) | `LabelDialog(["1", "2", "3"])`, `Generate PDF` | result `{total: 3, rendered: 2, marked_printed: 1, reprinted: 1, failures: [{product_id: "3", sku: "TEST-3", title: "Test lamp", message: "Barcode: no barcode."}], artifacts: [a1], failed_product_ids: ["3"]}`; `SAVED("/h/Documents/Session Manager Labels/a.pdf")` | one `labelsDownload`; the dialog stays open with the title `2 of 3 labels generated`; the lines `✓ 1 marked Printed`, `↻ 1 reprinted · stay Printed`, `✕ 1 failed`; one card for product 3 (`TEST-3`, `Test lamp`, the picture of the check's item 3) with the red pill `Can't generate: Barcode: no barcode.`; `Failed products stay selected so you can fix and retry.`; `Saved and opened in Preview: /h/Documents/Session Manager Labels/a.pdf`; the only button is `Close`; `toast.success` not called; `Close` → `onClose` received the result | 09C §14.2 ("When anything went wrong"); M-L4; §12.4 |
| L4·C9(b) | `LabelDialog(["1", "2", "99"])`, `Generate PDF` | the check answered `unresolved: [{product_id: "99", sku: null, …}]`; result `{total: 2, rendered: 2, marked_printed: 1, no_price: 1, artifacts: [a1]}`; `SAVED` | the file is saved (one `labelsDownload`); the dialog stays open: title `2 of 3 labels generated`, `○ 1 stay Not printed · No selling price`, `99 · Not found in Pricing`, `Close`; `toast.success` not called | 09C §14.2 ("no not-found product"); M-L4 |
| L4·C9(c) | `LabelDialog(["3"])`, `Generate PDF` | result `{total: 1, rendered: 0, failures: [one, message "Barcode: no barcode."], artifacts: []}` | no `labelsDownload`; title `Label not generated`; the failure's card with its red pill; `Close`; no `Failed products stay selected…` (one product) | 09C §14.2; M-L4 |
| L4·C9(d) | after `labelsGenerate` rejected with `labels_unreachable`: press `Cancel` | `labelsPartial` answers `{total: 347, rendered: 300, marked_printed: 300, not_run: <47 ids>, artifacts: [a1]}`, `SAVED`; on another render it rejects with `not_found` | one `labelsPartial({check_id})`, then exactly one `labelsDownload` for `a1`; title `300 of 347 labels generated`; the line `✕ 47 not generated`; `Close`; no toast. With `not_found`: the dialog closes with `null` and `labelsDownload` was never called | MC-L2.10; 09C §14.2 ("the files that were made are still saved and opened"); M-L4 (replaces L3·C14(d), L3·C15(e)) |
| L4·C9(e) | `LabelDialog(["1"])`, `Generate PDF`; then `Try again` | a fully generated result with one file; `labelsDownload` rejects with `labels_save_failed` (message `Couldn't save the PDF: disk full.`), then `SAVED("/h/…/a.pdf")` | the dialog stays open with the title `Label generated`, the message `Couldn't save the PDF: disk full.` and `Try again` beside it; no toast; the press makes a second `labelsDownload` with the same arguments; then the line `Saved and opened in Preview: /h/…/a.pdf`, no `Try again`, the dialog still open until `Close`; `toast.success` never called | 09C §14.2 ("a Try again next to a file that couldn't be saved"); §10 |
| L4·C9(f) | same start | `labelsDownload` rejects with `labels_expired` (message `This file has expired. Generate the labels again.`) | that message, no `Try again`, `Close` | 09C §10 |
| L4·C9(g) | same start | `labelsDownload` answers `{path: "/Users/x/Documents/Session Manager Labels/a.pdf", opened: false}` | the dialog stays open with `Saved, but Preview couldn't open it: /Users/x/Documents/Session Manager Labels/a.pdf`; no `Try again`; exactly one `labelsDownload` in the whole test; no toast | 09C §14.2 ("Preview opens each file once … the dialog says where") |
| L4·C9(h) | `LabelDialog(["1", "2"])`, `Generate PDF` | a fully generated result with files `a1`, `a2`; `labelsDownload` rejects for `a1` with `labels_save_failed` and is `SAVED` for `a2` | both were asked (two calls, `a1` then `a2`); `a1`'s line has the message and `Try again`, `a2`'s line reads `Saved and opened in Preview: …`; title `2 labels generated`; `Print at 100 % (not Fit to page).`; no toast | 09C §14.2 ("the files that were made are still saved and opened") |

### Delegated choices
The look of the busy line and of a file's line · whether the automatic downloads are driven by an effect or by the promise chain of `generate` / `leave` (each file once either way) · whether `LabelDoneView` keeps its name.

### Notes
- Hazard (M-L3, M-L2): this unit must not touch how Generate is called. `labelsGenerate` is still called once per press with the check id; Try again after a failed Generate is L3·C14(b), unchanged. Only what happens **after** a result changes.
- Hazard: "each file once" must survive React's re-renders and a Try again of Generate: the files of chunks finished before a failed chunk arrive again in the final result, and a file already asked must not be asked again (it would save a second numbered copy; L3 backlog note). Key the ref by artifact id.
- Hazard: a `labelsPartial` result's files are saved and opened too (09C §14.2), exactly like a full result's.
- Hazard: the toast is raised by the dialog's logic (one place), so every host (session row ⋯, session page ⋯, Labels row ⋯, the batch bar) gets it; hosts must not add their own.
- Hazard: a saved file's path is outside the data folder: plain text, never `previewUrl` (L3 hazard).
- `labels_download` and `LabelsService::download` are unchanged; `l3_labels_download::` and the desktop `l3_tests` rows L3·C11(d), (h), (i) stand.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/labels src/features/sessions src/features/shell` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time, never in the background, never piped.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-integrations --lib conversations::beyo_pricing::labels` · `cargo test -p psm-testkit --test suite h4_details_only_publish::` · `cargo test -p psm-testkit --test suite l4_labels_check_cards::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: the earlier tests changed are exactly those named under each unit's "Earlier tests" (`h4_c9a` replaced; `l1_c7c`'s note assertion; `l3_c11a`'s `picture_path: null`; `l3_c12b` and `l3_c14d` removed; `l3_label_done_dialog.test.tsx` deleted; fixtures and mocks only elsewhere) · `derive_status`, `row_label` and `ready_session_ids` are untouched and H4·C9(b)–(g) pass unchanged · both session commands still attach `.with_remote` · `provider.run` is still called at exactly one place and U3 changed no Rust file · `labelsDownload` is called automatically at most once per file and never for a file saved but not opened · the toast is raised at one place · no `Download PDF` and no `Done` remain in `apps/desktop/src/features/labels` · the CSS gives the dialog a fixed height, the list `overflow-y: auto`, the footer outside it, the thumbnail a 1:1 box and the title and group lines one line with an ellipsis (jsdom cannot prove these) · core names no platform and no provider code (H7) · no migration and no column (`SqliteStore::migrations()` unchanged, M-L1) · the fixture sentence of L4·C3 has the form cited under U2 "Rule 17" · no company data in fixtures. Notes go to `docs/implementation/backlog_labels/L4_owner_corrections.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Needs the Pricing key with `labels:read` and `labels:write`. **Generating marks products Printed in Pricing.**
1. Sessions list: a product that was photographed on the other Mac and opened here once now sits under **Published** with "Archived in Shopify"; the Open number no longer counts it.
2. Labels box → tick three or more products → **Generate labels for N**. The dialog keeps its size; the list of products scrolls while the summary and **Cancel / Generate PDF** stay at the bottom.
3. Each product is a card: picture on the left with the SKU under it, the title on one line (hover for the full title), price and what will happen, and under it small pills only when something needs saying ("No price", "Can't generate: …", "Not published yet").
4. On a product that is not published yet, press **QR link**: the address the QR code opens appears under the pills; press again to hide it; clicking the address opens the browser.
5. Click another card: its label preview shows on the right.
6. **Generate PDF**: the PDF opens in Preview by itself, the dialog closes and a note says "N labels generated · opened in Preview". Print with ⌘P at 100 %.
7. A batch with a product that cannot be generated: the files that were made still open in Preview; the dialog stays and shows "2 of 3 labels generated" with the problem as a card, and that product is still ticked after **Close**.
8. One label from a session's ⋯ → Generate label → Generate PDF: Preview opens, "Label generated · opened in Preview".

## Review log
- **2026-11-25 implement r1 (Claude Opus 5.5, `pipeline-implementer`).** Built the three units in order with a checkpoint
  each: U1 `9cda2589` (a `Remote(ShopOnly)` row labelled Archived has status Published in `list` / `page`, private
  `row_status` in `services/sessions.rs`; label computed before status; `derive_status`, `row_label`,
  `ready_session_ids` untouched; `h4_c9a` replaced by L4·C1(a)), U2 `7600a70b` (`NoteKind` / `PlanNote`,
  `LabelCheck.pictures`; adapter `plan_note`; `LabelsService` remembers SKU + `thumb_url` per row from `list` and
  `select_all`, `check_pictures` from one `SessionsService::list(All)` else `search_picture`; check item DTO `notes`
  / `picture_path`, bindings regenerated; `FakePricing::note_label`; `LabelCard`, two-line group header, scrolling
  list + pinned footer; `l3_c12b` removed, `l1_c7c` note assertion and `l3_c11a` `picture_path: null` updated), U3
  `5d25b5f3` (`useLabelDialog` `saving` phase: every file of a result saved and opened once, toast + close when
  everything generated, else the result view with problem cards, file lines, Try again, Close;
  `l3_label_done_dialog.test.tsx` and `l3_c14d` removed). Assumptions and deviations: handoff
  `handoffs/implementer/handoff_L4_owner_corrections_implement_r1.md` (notably: `LabelCard` gained a `failure` prop
  for U3's problem cards; the unused `PRICE`, `T50` and `result` fixtures of `l3_label_check_dialog.test.tsx` were
  removed with the tests that used them, for `tsc`'s unused-locals rule). Closing gate green.
