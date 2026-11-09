---
phase: P60_take_back_and_self_healing
title: The automatic take-back runs cleanly (Capture One is not reopened early, dialogs and a stuck camera service are named), and photos whose files were deleted outside the app never break the flow — no endless renders, no blocked take-back, no re-render storm after Capture One reopens; recipes repair themselves before use
difficulty: high
projection: no
depends_on: [P59_item_type_from_shopify_product_type]
---
# P60 — Take-back fixes (#22, #23, #25, false failure) and self-healing for deleted files (#24, #26) + recipe self-repair (10F §5)

Written by the orchestrator (Opus) on 2026-11-09 from the owner's hands-on check of P57–P59 (evidence `docs/intention/evidence/sony_camera_lab.md`
§Hands-on check 2026-11-09 evening). The owner: "I was removing a lot of pictures managing my storage … if I manually delete a picture this shouldn't
break the flow of the app." Book changelog entry 43.

## What the check showed
- **Automatic take-back works** after a Mac restart: plug-in → cable session in 12.5 s with no click (`ptpcamerad` claim at +0 s, helper sees it at
  +4.7 s, Capture One quit at +6.5 s, `ptpcamerad` released at once, our USB session at +12.5 s).
- **#22** Capture One was relaunched **0.34 s** after the take-back quit it (stopwatch 18:46:54.661 → 18:46:54.998), before our USB session; earlier the
  same happened after the photographer quit it by hand (17:53:01.988 → 17:53:02.319). Likely path: queued preview renders call Capture One through the
  editor gate, which launches it. The take-back's step 3 then reported **"Capture One did not quit"** although it had (false failure).
- **#23** With Capture One's sign-in window open, the take-back's quit timed out (30 s) / "did not quit (a dialog may be open)" — only in the log; the
  photographer saw "Capture One is holding the camera's USB connection" with no reason.
- **#25** After the owner had killed `ptpcamerad` by hand that morning (outside the app), the respawned service held the cable with **no app running**
  (18:04:14–18:04:26: no Session Manager, no Capture One, cable held). Every take-back failed; the header kept blaming Capture One. A Mac restart fixed it.
- **#24/#26** The owner deleted many RAWs by hand (storage). Since then ~120 photos whose files are gone are re-requested from Capture One: 529 render
  log lines, `variant_not_found` (129 deferred, 63 refused) and `provider` (52) errors after each Capture One relaunch. This storm kept the take-back at
  "Waiting to take the cable: a preview is rendering", and a new cable photo (delivered to the Mac 1.4 s after exposure) appeared in Session Manager only
  after ~20 s, behind the queue. `recipe_missing: PSM Preview` (173×) appeared after Capture One's settings were erased, until a restart re-provisioned.

## Goal
1. **No early relaunch (#22).** While a take-back (automatic or manual) is between its quit step and its end, nothing launches Capture One: render
   requests and any other editor call wait (they do not launch, fail or count attempts). The same applies after the photographer quits Capture One while
   the camera is on the cable and our helper does not hold it (P54's wait). The take-back reports success when its outcome is our USB session with
   Capture One back — never "did not quit" when Capture One did quit.
2. **Say why it waits (#23).** If Capture One does not answer or does not quit (timeout / still running after the quit), the header says
   `Capture One has a window open — close it so Session Manager can take the camera`; the automatic take-back retries for the same plug-in as soon as
   Capture One answers again.
3. **Stuck camera service (#25).** If, after Capture One has quit (take-back step), the cable is still held for 10 s, the header says
   `macOS's camera service is holding the cable with no app using it. Restart the Mac to free it.` (instead of naming Capture One).
4. **Deleted files heal themselves (#24/#26; owner 2026-11-09, changelog 43).** A photo whose file no longer exists on disk is marked **missing**: it
   leaves the session views like a deleted photo (classification unchanged in history; a reason `file_missing`), no render is ever requested for it, it
   never counts toward "a preview is rendering", and Health shows `N photos' files were deleted outside Session Manager`. If the file comes back (same path),
   the photo returns as it was. The check runs at startup, when Capture One (re)opens, and whenever Capture One answers `variant_not_found` for a photo.
5. **No re-render storm.** When Capture One (re)opens, previews are requested only for photos that need one (no current preview, or a real adjustment,
   P56); photos with a current preview are not re-requested. A new arrival's preview is never queued behind photos that cannot render.
6. **Recipes repair themselves before use (10F §5).** Before each render batch the app checks its recipes (`PSM Preview`, `PSM Master`) and repairs them
   (as at startup); a `recipe_missing` or wrong-format answer triggers one repair and one retry, not a deferral loop.

**Not in this phase:** the rest of 10F (duplicates clean-up, preview-export clean-up, retention after publish, storage budget) → P61/P62.

## Read first
Evidence §Hands-on check (2026-11-09 evening) and §Acceptance run 2; book `10E` MC-34 (§Take it back), changelog 38, 43; `10F` §5; `10A` (render contract,
MC-16/17), `05_domain_model.md` (photos, classification). P54, P56, P57, P58 narratives and Review logs. Code: `crates/integrations/src/camera/direct/{take_back.rs,
start_order.rs, ownership.rs}`, `crates/integrations/src/camera/runtime.rs` (automatic take-back), `crates/integrations/src/editing/capture_one/{gate.rs,
launch_gate.rs, adapter.rs, renders.rs, provision.rs}`, `crates/core/src/services/{previews.rs, sessions.rs, photos*.rs}`, `crates/core/src/mechanisms/camera_status.rs`,
Health (`crates/core/src/services/health*.rs`), the header camera popover. Tests: p48, p54, p56, p57, p58 testkit files; fake Capture One.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p60_take_back_and_self_healing.rs` (new); a migration if `file_missing` needs a column (keep the P01 schema
test in step); UI test for the new header sentences; fake Capture One extensions (relaunch, unanswered quit, variant_not_found). ≈ 22 paths, ≈ 800 lines.

## Core mechanisms (unit tests allowed)
"Missing file" decision; render-request eligibility (missing, current preview, take-back hold).

## Criteria (8 criteria, 9 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | no launch during take-back | take-back past its quit step; 5 queued preview renders | the fake Capture One receives **no** launch until our USB session is reported (or the take-back fails); then exactly one launch; the renders run after it | `[rule]` MC-34; #22 |
| C1(b) | no false failure | as C1(a), outcome USB session + Capture One back | take-back result success; no "did not quit" anywhere | `[rule]` MC-34; #22 |
| C2(a) | dialog named, retried | fake Capture One ignores quit (dialog) | header `Capture One has a window open — close it so Session Manager can take the camera`; when the fake answers again, the automatic take-back runs (same plug-in) | `[rule]` changelog 38; #23 |
| C3(a) | stuck camera service | quit done, `usb.claimed_by` continues 10 s | header `macOS's camera service is holding the cable with no app using it. Restart the Mac to free it.` | `[rule]` MC-34 §Guide; #25 |
| C4(a) | deleted file heals | session with 3 photos; delete one RAW on disk; app start (and separately: Capture One answers `variant_not_found` for it) | the photo is missing (not in the session view), Health `1 photos' files were deleted outside Session Manager`, no render request for it ever; restoring the file brings it back | `[rule]` changelog 43 |
| C4(b) | missing files never block | 50 photos missing, one photo processing nothing; automatic take-back | take-back proceeds (no "a preview is rendering") | `[rule]` changelog 43; MC-34 |
| C5(a) | no storm on reopen | 100 photos with current previews; Capture One relaunches | zero preview requests for them; a new arrival's request is the first sent | `[rule]` P56/P58 queue; #26 |
| C6(a) | recipe self-repair | fake Capture One reports `PSM Preview` missing / exports TIFF | one repair call, one retry; preview produced; no deferral loop | `[rule]` 10F §5 |
| C7(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, sidecar stub tests green | `[rule]` master plan §7.2 |

## Delegated choices
Where the take-back hold lives (gate vs runtime); how "missing" is stored (column vs derived at load) as long as the photo's history is kept; Health wording
beyond the sentence above; the cadence of the 10 s stuck-service check.

## Manual try-out
Plug in with Capture One open: Capture One closes, the dot reads Cable, and only then Capture One reopens (≈ 7–8 s total). Delete one RAW by hand in
Finder: the photo disappears from the session with a Health note; Capture One's preview window does not loop. Open a dialog in Capture One and plug in:
the header asks you to close it, and the take-back runs once you do.

## Notes
- **Hard rules (owner):** never kill or quit any process except Capture One through the take-back; never delete photos or files (missing means the
  photographer already deleted them — the app only stops asking for them); the suite never needs the camera or Capture One. Default implementer (Codex).
  If the SDK is present locally, prove the SDK build compiles, then restore the stub; never commit build output.

## Review log
