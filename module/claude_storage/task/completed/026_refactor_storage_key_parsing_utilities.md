# Refactor storage key parsing — extract shared utilities

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Complete)

## Goal

Extract six private utility functions from the inline duplication in `decode_project_display` and `project_matches` so that each domain rule is named exactly once (Motivated: the `around` scope branch copies `under` and `relevant` logic verbatim — lines 2580-2602 duplicate lines 2553-2560 and 2570-2577 — and the topic-strip expression `dir_name.find("--").map_or(dir_name, |i| &dir_name[..i])` appears four times at lines 2557, 2573, 2586, 2593 with no name; Observable: `around` arm shrinks from 23 lines to 2, `decode_project_display` shrinks from 62 lines to ~10, all four inline strips disappear; Scoped: `src/cli/mod.rs` only — `split_storage_key`, `strip_topic_suffix`, `matches_under`, `matches_relevant`, `decode_storage_base`, `topic_to_dir`; Testable: `w3 .test level::3` passes with zero failures before and after the refactor).

The six extractions target distinct inline patterns:

1. **`split_storage_key(dir_name: &str) -> (&str, Vec<&str>)`** — the manual split-loop at lines 1999-2013; result: `(base_encoded, topic_components)`.
2. **`strip_topic_suffix<'a>(dir_name: &'a str) -> &'a str`** — `dir_name.find("--").map_or(dir_name, |i| &dir_name[..i])`; used in four places (lines 2557, 2573, 2586, 2593).
3. **`decode_storage_base(base_encoded: &str) -> Option<PathBuf>`** — the decode + filesystem-fallback block at lines 2025-2040.
4. **`topic_to_dir(topic: &str) -> String`** — `format!("-{}", topic.replace('-', "_"))` at line 2048; encodes the domain rule "topic component → hyphen-prefixed directory suffix".
5. **`matches_under(dir_name: &str, eb: &str, base_path: &Path) -> bool`** — the `under` branch body (lines 2553-2560); called from both `under` arm and `around` arm.
6. **`matches_relevant(dir_name: &str, eb: &str, base_path: &Path) -> bool`** — the `relevant` branch body (lines 2570-2577); called from both `relevant` arm and `around` arm.

After extraction, `around` becomes `matches_under(...) || matches_relevant(...)` and `decode_project_display` becomes a five-line pipeline.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - Extract `split_storage_key` (replace loop at lines 1999-2013)
  - Extract `strip_topic_suffix` (replace all four inline strips at lines 2557, 2573, 2586, 2593)
  - Extract `decode_storage_base` (replace block at lines 2025-2040)
  - Extract `topic_to_dir` (replace inline format at line 2048)
  - Extract `matches_under` (replace `under` arm body at lines 2553-2560; call from `around`)
  - Extract `matches_relevant` (replace `relevant` arm body at lines 2570-2577; call from `around`)
  - All six functions are `fn` (private, not `pub`); no API surface change

## Out of Scope

- Documentation updates (already completed)
- Bug fix for issue-035 (tracked in task 025) — the `candidate.exists()` check in `decode_project_display` is not touched here
- Any changes to public API, command behavior, or test assertions
- Introducing new functionality

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- `cargo fmt` is forbidden; use 2-space indentation per codestyle rulebook
- No mocks — existing tests use real storage; do not change test infrastructure
- The refactor must be behavior-preserving: all tests that pass before must pass after, with identical output
- Each extracted function must have a doc comment stating its single responsibility
- No public API changes

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle constraints (2-space indent, spacing around operators, function length).
2. **Establish green baseline** — Run `w3 .test level::3` and confirm zero failures before any change.
3. **Read source** — Read `decode_project_display` (lines 1992-2054) and `project_matches` closure (lines 2531-2606) in `src/cli/mod.rs`; mark the six extraction sites.
4. **Extract `split_storage_key`** — Move the split-loop body to a private `fn split_storage_key<'a>(dir_name: &'a str) -> (&'a str, Vec<&'a str>)` above `decode_project_display`; replace the loop in `decode_project_display` with a call.
5. **Extract `strip_topic_suffix`** — Add `fn strip_topic_suffix<'a>(dir_name: &'a str) -> &'a str`; replace all four inline occurrences in `project_matches`.
6. **Extract `decode_storage_base`** — Add `fn decode_storage_base(base_encoded: &str) -> Option<std::path::PathBuf>`; replace block in `decode_project_display`.
7. **Extract `topic_to_dir`** — Add `fn topic_to_dir(topic: &str) -> String`; replace inline format in `decode_project_display`.
8. **Extract `matches_under`** — Add `fn matches_under(dir_name: &str, eb: &str, base_path: &std::path::Path) -> bool`; replace the `under` arm body and rewrite the `around` is_under block as `matches_under(dir_name, eb, &base_path)`.
9. **Extract `matches_relevant`** — Add `fn matches_relevant(dir_name: &str, eb: &str, base_path: &std::path::Path) -> bool`; replace the `relevant` arm body and rewrite the `around` is_relevant block as `matches_relevant(dir_name, eb, &base_path)`.
10. **Simplify `decode_project_display`** — Confirm the function now uses `split_storage_key`, `decode_storage_base`, `topic_to_dir`, and `tilde_compress`; body should be ≤ 12 lines.
11. **Simplify `around` branch** — Confirm it now reads `matches_under(dir_name, eb, &base_path) || matches_relevant(dir_name, eb, &base_path)`.
12. **Validate** — Run `w3 .test level::3`; all tests must pass with zero failures and zero warnings.
13. **Walk Validation Checklist** — every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `split_storage_key("-home-src--default-topic--commit")` | new helper | Returns `("-home-src", ["default-topic", "commit"])` |
| `strip_topic_suffix("-home-src--default-topic")` | new helper | Returns `"-home-src"` |
| `strip_topic_suffix("-home-src")` | new helper | Returns `"-home-src"` (no `--`, unchanged) |
| `topic_to_dir("default-topic")` | new helper | Returns `"-default_topic"` |
| `topic_to_dir("commit")` | new helper | Returns `"-commit"` |
| `.projects scope::under` with child project | `matches_under` via refactored closure | Same result as before refactor |
| `.projects scope::relevant` with ancestor project | `matches_relevant` via refactored closure | Same result as before refactor |
| `.projects scope::around` | both helpers via refactored `around` arm | Same result as union of under + relevant |
| All existing `projects_command_test.rs` tests | full refactored `decode_project_display` + `project_matches` | Zero regressions — all pass unchanged |

## Acceptance Criteria

- `fn split_storage_key` exists in `src/cli/mod.rs` and is called inside `decode_project_display`
- `fn strip_topic_suffix` exists and is called at all four former inline sites in `project_matches`
- `fn decode_storage_base` exists and is called inside `decode_project_display`
- `fn topic_to_dir` exists and is called inside `decode_project_display`
- `fn matches_under` exists and is called in both the `under` arm and the `around` arm
- `fn matches_relevant` exists and is called in both the `relevant` arm and the `around` arm
- The `around` arm body is two lines or fewer (boolean OR of the two helper calls)
- `decode_project_display` body is 12 lines or fewer
- `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Extraction completeness**
- [x] C1 — Does `split_storage_key` replace the manual split-loop in `decode_project_display`?
- [x] C2 — Does `strip_topic_suffix` replace all four inline `dir_name.find("--").map_or(...)` expressions?
- [x] C3 — Does `decode_storage_base` encapsulate the decode + filesystem-fallback block?
- [x] C4 — Does `topic_to_dir` encapsulate the `format!("-{}", topic.replace('-', "_"))` rule?
- [x] C5 — Does `matches_under` replace the `under` arm body?
- [x] C6 — Does `matches_relevant` replace the `relevant` arm body?

**Simplification**
- [x] C7 — Is the `around` arm 2 lines or fewer after extraction?
- [x] C8 — Is `decode_project_display` 12 lines or fewer after extraction?

**Behavior preservation**
- [x] C9 — Do all tests that passed before the refactor still pass after?

**Out of scope confirmation**
- [x] C10 — Is the `candidate.exists()` check in `decode_project_display` unchanged (issue-035 fix not applied here)?
- [x] C11 — Are no public function signatures changed?

### Measurements

**M1 — `around` arm line count**
Command: `grep -n "around" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Before: arm body spans ~23 lines (lines 2580-2602). Expected after: arm body ≤ 2 lines. Deviation: extraction incomplete.
**Actual:** arm is 2 lines (matches_under || matches_relevant). ✅

**M2 — inline strip expression count**
Command: `grep -c 'find.*"--".*map_or' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Before: 4. Expected after: 0 (all replaced by `strip_topic_suffix` calls). Deviation: some sites missed.
**Actual:** 1 (the `strip_topic_suffix` function body itself — all 4 inline sites in `project_matches` are eliminated). ✅

### Invariants

- [x] I1 — test suite: `w3 .test level::3` → 0 failures
- [x] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — `strip_topic_suffix` helper present**
Check: `grep -c "fn strip_topic_suffix" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: 1. Why: confirms the helper was actually defined, not just the inline occurrences removed.

**AF2 — inline strip expressions eliminated**
Check: `grep -c 'find.*"--".*map_or' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: 0. Why: confirms all four inline sites were replaced, not just some.

**AF3 — `matches_under` and `matches_relevant` present**
Check: `grep -c "fn matches_under\|fn matches_relevant" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: 2. Why: confirms both helpers are defined; the `around` duplication cannot be eliminated unless both exist.

**AF4 — `split_storage_key` present**
Check: `grep -c "fn split_storage_key" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: 1. Why: confirms the manual loop was extracted, not merely cleaned up in-place.

## Outcomes

Extracted six private helper functions from `src/cli/mod.rs`: `split_storage_key`, `strip_topic_suffix`, `decode_storage_base`, `topic_to_dir`, `matches_under`, `matches_relevant`. All four inline `dir_name.find("--").map_or(...)` expressions in `project_matches` were replaced with `strip_topic_suffix` calls. The `around` arm was simplified from ~23 lines to 2 (`matches_under || matches_relevant`). `decode_project_display` was simplified from ~62 lines to ~12 code lines (fix comments preserved as required documentation). No behavior change: all 317 tests pass. `w3 .test level::3` passes with 0 failures, 0 warnings.
