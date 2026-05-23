# Run brief — P8.T2 + P8.T7 — Popup title-bar collapse-chevron flips

**Status:** Queued, not yet dispatched.
**Authored:** 2026-05-23 by the orchestrator, against `overhaul/infinity_orchestrator` HEAD `0a97cf3` (the post-merge state of PR #11 — SPEC §13.12b + Phase 8 P8.T14 are present).
**Reason for queuing:** the project's `plan-implementer` agent at `.claude/agents/plan-implementer.md` is not currently discoverable as a `subagent_type` in this Claude Code session (Glob-style traversal fails on the `.claude` junction; skills load via a different code path; agents do not). The `.claude/` layout is being restructured to fix this; once the agent is discoverable, the next orchestrator picks up this brief and dispatches.

**Target branch:** `feat/popup-collapse-chevrons` — create this branch off the current `overhaul/infinity_orchestrator` HEAD at dispatch time. (An earlier attempt's branch was deleted; the brief's content does not depend on that branch existing yet.)

**Verify before dispatching:** confirm `overhaul/infinity_orchestrator` HEAD hasn't drifted in a way that invalidates the assumptions below. If the six BIO popup files in scope have been refactored, re-validate the line numbers against the current file contents.

---

## Dispatch this brief to the plan-implementer (or a `general-purpose` agent acting as one)

The agent runs `plan-implementer`'s role contract (`.claude/agents/plan-implementer.md`); if that subagent_type still isn't discoverable, fall back to `general-purpose` with this brief and an instruction to follow that contract.

### Required reading (in order)

1. `.claude/agents/plan-implementer.md` — role contract; binding.
2. Invoke `/code-hygiene` at session start — the shared skill carries clippy pedantic + nursery, no-`#[allow]`, concise-rustdoc, no-paper-trail-pointer-comments rules. Compliant code on the first draft, not catch-at-gate.
3. `.claude/spec-authority.md` — directive doctrine + Current-project block.
4. `infinity_orchestrator/SPEC.md` — read §1 CRITICAL DIRECTIVE (especially carve-out #2) + §10 "Collapse chevron global popup pattern" / "Collapse direction".
5. `infinity_orchestrator/plan/phase-08-popup-reskins-polish.md` — read **only** `### P8.T2`, `### P8.T7`, the "File inventory → New files" entry for `popup_collapse_anchor.rs`, and the "BIO files needing allowed mild refactor — carve-out #1 + #2" table rows for the six popup files in scope. Do NOT execute anything outside P8.T2 + P8.T7.
6. `infinity_orchestrator/HANDOFF.md` — current project state.
7. The 6 BIO popup files cited by P8.T2 — read fully so you can confirm the line numbers haven't drifted:
   - `src/ui/step2/compat/compat_window_step2.rs`
   - `src/ui/step2/prompt/prompt_popup_step2.rs`
   - `src/ui/step2/update_check/update_check_popup_step2.rs`
   - `src/ui/step2/update_check/update_check_popup_source_editor_step2.rs`
   - `src/ui/step1/github_auth_popup_step1.rs`
   - `src/ui/step5/content/content_cancel_step5.rs`

### Tasks in scope

#### P8.T2 — eight single-line `.collapsible(false)` → `.collapsible(true)` flips (carve-out #2)

Per the plan's exact line citations at brief-authoring time:

- `compat_window_step2.rs:16` — main popup (1 flip)
- `prompt_popup_step2.rs:27` + `:123` — main popup + toolbar variant (2 flips)
- `update_check_popup_step2.rs:67` + `:589` + `:631` — main popup + confirm-latest-fallback + forks popup (3 flips)
- `update_check_popup_source_editor_step2.rs:20` — source editor (1 flip)
- `github_auth_popup_step1.rs:18` — GitHub OAuth (1 flip)
- `content_cancel_step5.rs:19` — Step 5 cancel-confirm modal (1 flip)

**Total: 8 single-keyword flips across 6 files.** Pure chrome change — no signature, no body, no behavior, no logic.

**Validate line numbers BEFORE editing.** Read each file, confirm the `.collapsible(false)` call is at or near the cited line. A small drift (a few rows) is fine — use the file content as authority. If a flip target is genuinely absent (call renamed, file restructured), STOP and emit `PLAN GAP` in the final report.

#### P8.T7 — verify anchor-on-collapse behavior

After applying the P8.T2 flips, empirically test each popup's collapse behavior. Three branches:

- **(a) egui anchors top-Y properly** when the body collapses → P8.T7 deliverable is "verified — no wrapper needed". Report this.
- **(b) egui re-centers vertically AND the popup invocation site is orchestrator-owned** (the popup `.show()` call lives in `src/ui/orchestrator/` or another redesign-owned file) → create new `src/ui/orchestrator/widgets/popup_collapse_anchor.rs` per the P8.T7 sketch (captures `Window::current_pos` at the collapse transition, pins the window's top-Y). Apply to the affected orchestrator-side invocation sites.
- **(c) egui re-centers vertically AND the popup invocation site is in a BIO file** → per SPEC §10's fallback decision tree, accept egui's native behavior. Threading the wrapper through a BIO invocation site would exceed carve-out #2's scope. Do NOT add the wrapper. Report this clearly so the orchestrator can decide whether to escalate for a new carve-out.

### NOT in scope

- P8.T1 (theme-token additions) — unless the wrapper genuinely needs one, which it shouldn't (positioning logic only).
- P8.T3+ (carve-out #1 + #6 color swaps).
- ANY other Phase 8 task.
- ANY edit to the touched BIO files OTHER than the 8 single-keyword flips.
- P8.T14 (per-modlist data ownership refactor — separate, larger work).

### Prep already settled

- Both binaries currently build clean on `overhaul/infinity_orchestrator` HEAD `0a97cf3` (verify still true at dispatch time).
- The 6 carve-out #2 file paths + line-number citations are in the phase-08 plan.
- The egui collapse behavior is the known unknown P8.T7 explicitly addresses — test, don't predict.

### Constraints

- **Net-new only EXCEPT the 8 carve-out #2 single-keyword flips named above.** The only other allowed new file is `src/ui/orchestrator/widgets/popup_collapse_anchor.rs` IF P8.T7 branch (b) fires.
- **Read `/code-hygiene` at session start.** Clippy pedantic + nursery, no `#[allow]`, concise rustdoc, no SPEC-§/phase/run/fix-set pointers in code comments. First-draft compliant.
- **Symbol-glyph rule.** The collapse chevron is egui's native title-bar widget — do NOT paint a custom glyph. If you find yourself rendering a custom string, stop and re-read SPEC §10.
- **No `#[allow]` in touched files** — fix the underlying issue if clippy complains.
- **Clean up scratch artifacts before reporting.** Delete `target_docs/` (cargo doc cache), throwaway probe bins under `src/bin/`, debug log files, scratch test fixtures, anything generated that's not part of the staged deliverable. Clean working tree is part of the deliverable (per `feedback_agents_clean_own_artifacts`).

### Verification gate (manual breakpoint, before reporting)

1. `cargo build --bin BIO --release` — must succeed.
2. `cargo build --bin infinity_orchestrator --release` — must succeed.
3. `cargo test --lib` — must pass; record the count (baseline ~515/515 per HANDOFF; confirm actual).
4. **Manual test** of each popup surface: launch the orchestrator binary, drive each popup, observe:
   - Title-bar chevron present?
   - Clicking the chevron collapses the body?
   - Title bar stays put OR re-centers vertically? (P8.T7 test — record per popup.)
5. **Render-gate PNGs via egui_kittest**: for each popup, an expanded-state PNG and a collapsed-state PNG, at shell width 1280px. Save under `target/render_gate/popup_collapse_chevrons/`. ~14 PNGs total (7 popup surfaces × 2 states).

### DO NOT commit

Implement + verify + report. The orchestrator commits.

### Report structure (numbered, terse)

1. **Files modified.** One line per file + the flip change.
2. **Files created.** Net-new files (only `popup_collapse_anchor.rs` if P8.T7 branch (b)).
3. **P8.T2 line-number reconciliation.** For each of the 8 flips, the actual line edited.
4. **P8.T7 outcome per popup.** Branch (a) / (b) / (c) result; if (b), which orchestrator-side invocation sites got the wrapper.
5. **Build results.** `cargo build --bin BIO --release`, `cargo build --bin infinity_orchestrator --release`, `cargo test --lib` count.
6. **Render-gate PNG paths.** 14 PNGs under `target/render_gate/popup_collapse_chevrons/`.
7. **Scratch artifact cleanup.** `git status` output confirming only intended deliverables.
8. **Escalations.** Any `SPEC CONFLICT` / `PLAN GAP` / behavior-anomaly findings.

### Already built / cross-phase context

- **SPEC §10** establishes the collapse chevron as the global popup pattern (body collapses upward; title bar stays at top).
- **Carve-out #2** in SPEC §1 explicitly authorizes single-line `.collapsible(false)` → `.collapsible(true)` flips on existing `egui::Window` builders. Pure chrome flip — no body content, no signature, no behavior change. Existing carve-out, not new authorization.
- **Phase 1's theme + font surface** is set up (`redesign_tokens.rs`, `redesign_fonts.rs`). Don't touch.
- **Both binaries coexist.** Orchestrator binary loads redesign palette + fonts; legacy `BIO` works unchanged.
- **The 7 popup surfaces** render inside BIO files. The orchestrator invokes them via `render_shared_popups`. The task is to flip the `.collapsible(false)` calls inside those BIO files (carve-out #2 authorized).
- **PR #11 merged** at HEAD `0a97cf3` — SPEC §13.12b + Phase 8 P8.T14 are now in the spec and plan. P8.T14 is the per-modlist data ownership refactor; separate work, not in scope here.

---

## Notes for the next orchestrator picking this up

- **The `feat/popup-collapse-chevrons` branch was created locally then deleted** at brief-authoring time (it had no commits). Re-create it off the current `overhaul/infinity_orchestrator` HEAD before dispatching.
- **HEAD will have drifted** by the time this is picked up. Verify the 6 popup files + their cited line numbers still match. If they don't, re-validate during the agent's `Read` step (the brief tells the agent to do this anyway).
- **If the line numbers have drifted significantly** (e.g., the entire popup file structure changed), this brief may be partially stale. Re-read the Phase 8 plan's P8.T2 + P8.T7 sections against the current source state and update this brief before dispatching.
- **`/code-hygiene` was renamed or restructured?** If the `.claude/skills/` layout changed during the restructure, the skill name might differ. Verify the skill is still invokable as `/code-hygiene` before relying on it.
- **Agent discovery still broken?** If `plan-implementer` is still not in the available `subagent_type` list, fall back to `general-purpose` with this brief and an explicit instruction to follow `.claude/agents/plan-implementer.md`'s role contract. That's what the previous orchestrator did before stopping; the work itself is unaffected by the fallback.
- **PR workflow on completion.** Per the orchestrator skill: verify the agent's report independently, run scoped quality gates (clippy pedantic + nursery on touched files, rustfmt on staged files only, comment hygiene), commit on the run branch (Xgatt-only attribution, no Claude co-author), run the final pre-handoff rebuild gate (touched files are BIO `.rs`, so the gate fires — confirm `cargo build --bin infinity_orchestrator --release` is a no-op `Finished` with no `Compiling bio` line), push `-u origin feat/popup-collapse-chevrons`, open PR with `gh pr create --repo Xgatt/Born2BSaltys_Infinity_Orchestrator --base overhaul/infinity_orchestrator --head feat/popup-collapse-chevrons --title "feat: popup collapse-chevron flips (P8.T2 + P8.T7)" --body-file <body>`. Hand back to user with PR URL + the manual-test script (the 7 popups × 2 states).
