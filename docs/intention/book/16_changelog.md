# 16 — Shaping changelog

1. **2026-10-23 — Shaper.** First draft (single file) grounded on the Capture One 16.8.6 dictionary;
   official hook properties replace the undocumented `CO_CaptureDone`; milestones renamed S0–S8.
2. **2026-10-23 — Shaper.** `capture-one-mcp` not adopted; `approved` removed from image state;
   per-product capture folder proposed (D1); default group gallery.
3. **2026-10-23 — Owner.** uv runtime, FastAPI standard layout without the contract set, React +
   shadcn/ui, Node allowed, custom dev-app token, public repo.
4. **2026-10-24 — Owner (chat).** Review model: approve / condition / delete with undo; fast original
   preview then adjusted version with toggle; phone remote wanted.
5. **2026-10-24 — Owner (architecture memo).** Commercial direction; Tauri + Python sidecar;
   Items/Attributes/Photos/Sessions vocabulary; integration layer with capabilities; generic
   attribute schema; AI on the generic model; session completion independent of publishing;
   MVP scope, production architecture.
6. **2026-10-24 — Owner.** C1 public, C2 session per item, C3 schema now / agent later, C4 one
   transport, C5 Capture One only provider.
7. **2026-10-24 — Shaper.** Draft restructured into this book (map + chapters) at the owner's
   request so downstream sessions load only relevant chapters. Design handoff folded in:
   ready rule, status order, dirty state, undo timing, toggle, keyboard, field kinds, customize
   fields, batch publish, notifications, condition attribute = `custom.damage_report`.
   "Replace" flag retired in favour of the dirty-state re-publish (N1 opened). N2 and N3 opened.
   Measurement M7 (integration-free completion) added.
8. **2026-10-24 — Owner.** D1 yes; D2 mirror as an editing-provider capability, coloring likely
   not needed; N1 mirror; N2 exclude (yes to recommendation); N3 after the desktop MVP. D4 was
   misread as "Capture One retouch export" and is re-asked in plain words; the owner's real
   question (how the adjusted preview can be live, and reflect later edits) is answered in
   chapter 10 (adjusted hook + sidecar watcher, re-render through the preview recipe).
9. **2026-10-24 — Owner (second architecture memo).** Rust/Tauri replaces Python/FastAPI as the
   backend; Python optional; camera connectivity and editing software are integrations behind
   provider interfaces with capabilities; Shopify stays an optional commerce integration; layers
   frontend / core / services / integrations / infrastructure; Tauri commands thin; MVP
   integrations exactly Sony ZV-E10, Capture One, Shopify; validate packaging early.
10. **2026-10-24 — Shaper.** Memo folded in: chapters 01 (H3, H5, H7, H8), 02, 03, 04, 05, 06, 09,
    10 (renamed "Camera and editing providers"), 11, 12, 13, 14 (M2 → write-direction mirror),
    15 (R16–R25; R1, R2, R8, R10 superseded). Conflicts found and opened as cards: A1 (R8 "one
    transport" vs Tauri commands), A2 (camera provider vs Capture One owning the tether).
    `primary variant adjusted script` added to the hooks in use; VERIFY-8/9/10 added to the
    Capture One evidence. `.gitignore` and `CLAUDE.md` updated to the Rust stack.
11. **2026-10-24 — Owner.** A1 Tauri commands now, server later (R26). A2 camera through Capture
    One for the MVP; the physical camera link (USB or wireless) remains open and must not disturb
    the implementation (R27). D4 to be explained again.
12. **2026-10-24 — Owner.** D4 at publish (R28). Preview refresh rule: 15–20 s after the last edit
    in Capture One, overridden by the app window returning to the front (R29). Open-decision ledger
    empty; status set to READY_FOR_RATIFICATION and the ratification surface presented.
