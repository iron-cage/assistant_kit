# Fix `guard_once_pinned` doc comment — `resolved` param is advisory for alias specs

## Execution State

- **State:** ✅ (Completed)
- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **Priority:** 0
- **Value:** 5
- **Easiness:** 9
- **Safety:** 9
- **Advisability:** 0
- **Dir:** .
- **Validated By:** claude-sonnet-4
- **Validation Date:** 2026-05-24
- **Blocked Reason:** null
- **Closes:** null

## MOST Goal

- **Motivated:** BUG-001 (filed 2026-05-23) identified 5 fix locations. Locations 1–4 (markdown doc files) have been corrected. Location 5 — `src/commands.rs:625` `guard_once_pinned` doc comment — still says "compare installed vs preferred and restore on drift" with no mention of the alias re-resolution semantics or the advisory nature of the `resolved` parameter. Any developer reading only the source doc comment gets an incorrect mental model identical to the pre-IT-15 state.
- **Observable:** After this fix, the `guard_once_pinned` doc comment states: (1) that `resolved` is the stored `preferredVersionResolved` value, advisory for alias specs; (2) that the function re-resolves `spec` through the current alias table at call time; (3) that when `spec` is an alias, `resolved_now` (not `resolved`) is used as the target. Verified by reading the function at its source location.
- **Scoped:** Exactly one function (`guard_once_pinned`) in one file (`src/commands.rs`). Doc comment content only — no logic changes.
- **Testable:** `verb/test l::3` passes in Docker with no warnings; doc comment at `guard_once_pinned` accurately describes the advisory semantics as stated in Observable.

## Null Hypothesis

> The existing one-line doc comment is sufficient — developers can infer `resolved` semantics from the code.

Disproved: BUG-001 root cause analysis confirmed that the existing doc comment ("compare installed vs preferred and restore on drift") describes only the external contract without the critical internal nuance. The IT-15 fix was applied code-first; the doc comment was never updated. TC-410 in `mutation_commands_test.rs:917-918` is the only location with accurate semantics — invisible to source readers.

## In Scope

- `src/commands.rs` — `guard_once_pinned` function doc comment at approximately line 625; the existing one-line doc comment must be expanded to document that `resolved` (the `preferredVersionResolved` stored value) is advisory for alias specs and that the function re-resolves `spec` through `resolve_version_spec()` at call time, using the result as the target when the spec is an alias.

## Out of Scope

- Changes to `guard_once_pinned` logic (already correct per IT-15 fix); changes to any other function or file; updating `docs/` markdown files (BUG-001 doc fixes 1–4 already applied directly); adding new tests.

## Work Procedure

1. Open `src/commands.rs`, navigate to `guard_once_pinned` (~line 625).
2. Replace the single-line doc comment `/// Guard path for pinned versions: compare installed vs preferred and restore on drift.` with an expanded comment that documents: the `resolved` param is the stored `preferredVersionResolved` value; for alias specs the function re-resolves `spec` through `resolve_version_spec()` and uses `resolved_now` as the target; `resolved` is advisory for alias specs and authoritative only when `spec` is a concrete semver.
3. Run `verb/test l::3` in Docker (from the `claude_version` module root via `bash runbox/runbox .test`).
4. Confirm clean output — no warnings, no test failures.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| Doc comment source review | `guard_once_pinned` at `commands.rs:625` | Comment states `resolved` is advisory for alias specs; re-resolution via `resolve_version_spec()` is documented |
| Level 3 verification | Entire `claude_version` crate | `cargo nextest`, `cargo test --doc`, `cargo clippy` all pass with no warnings |

## Acceptance Criteria

- AC-1: `guard_once_pinned` doc comment mentions that `resolved` is advisory for alias specs.
- AC-2: Doc comment mentions that `spec` is re-resolved through the current alias table at call time.
- AC-3: `verb/test l::3` passes with zero warnings in Docker.

## Related Documentation

- `docs/feature/001_version_management.md` — Version guard behavior and `preferredVersionResolved` advisory semantics (BUG-001 fix locations 3–4, already applied)
- `docs/pattern/001_version_lock.md` — Layer 5 recovery signal semantics (BUG-001 fix locations 1–2, already applied)
- `task/claude_version/bug/001_preferred_version_resolved_doc_mismatch.md` — BUG-001 root cause and all 5 fix locations

## Outcomes

The `guard_once_pinned` doc comment in `src/commands.rs` was expanded from a one-line description to a three-point block documenting: (1) that `resolved` is the stored `preferredVersionResolved` value, advisory for alias specs; (2) that the function re-resolves `spec` through `resolve_version_spec()` at call time; (3) that for alias specs, `resolved_now` (not `resolved`) is used as the target. Level 3 verification passed (303/303 tests, 0 Clippy warnings). This was the final of five BUG-001 fix locations — locations 1–4 in the markdown docs had been applied separately. After this fix, every entry point for understanding `guard_once_pinned` semantics (source, tests, and docs) consistently describes the alias re-resolution contract.

### Validation Results

- AC-1: PASS — doc comment explicitly states `resolved` is advisory for alias specs.
- AC-2: PASS — doc comment states `spec` is re-resolved through the current alias table at call time.
- AC-3: PASS — Level 3 verification passes with zero warnings (303/303 tests, 0 clippy warnings).

## History

- **[2026-05-24]** `CREATED` — Fix `guard_once_pinned` doc comment to document advisory semantics of `resolved` for alias specs; completes BUG-001 fix.
- **[2026-05-24]** `COMPLETED` — Validated by claude-sonnet-4. Doc comment expanded at `src/commands.rs:625`; Level 3 verification passes (303/303 tests, 0 clippy warnings). BUG-001 fix location 5 complete.

## Verification Record

All 4 dimensions passed independent Agent subagent review (2026-05-24):

- **Scope Coherence:** PASS — In Scope names the specific doc comment location and required content; Out of Scope excludes logic changes, other files, and already-applied markdown fixes.
- **MOST Goal Quality:** PASS — Motivated (BUG-001 fix location 5, concrete undocumented semantics), Observable (3 specific statements the doc comment must contain), Scoped (`guard_once_pinned` in `commands.rs` only), Testable (`verb/test l::3` + doc comment content review).
- **Value / YAGNI:** PASS — Null hypothesis (comment sufficient, semantics inferrable) disproved by BUG-001 analysis; TC-410 is the only accurate semantics source and is invisible to source readers.
- **Implementation Readiness:** PASS — 4 numbered executable steps; Test Matrix has 2 rows; Acceptance Criteria AC-1–AC-3 present; Related Documentation references 3 files; History has CREATED event.
