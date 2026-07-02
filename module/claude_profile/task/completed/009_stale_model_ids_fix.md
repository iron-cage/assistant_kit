# Stale Model IDs Fix

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** agent
- **started_at:** 2026-07-02
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **dir:** src/ + module/claude_runner_core/src/
- **validated_by:** null
- **validation_date:** null

## Goal

The model IDs `claude-opus-4-6` and `claude-sonnet-4-6` are no longer accepted by the Claude API — the API rejects them with a model-not-found error. Any `clp .model set::opus` or `imodel::opus` invocation silently writes an invalid ID that causes the subsequent subprocess to fail at launch. This task corrects the string literals in all mapping functions and updates the test assertions that currently validate the stale values.

Replace stale model ID strings `claude-opus-4-6` and `claude-sonnet-4-6` with the current IDs `claude-opus-4-8` and `claude-sonnet-5` in all source code functions in `claude_profile` that map model shorthands or resolve subprocess models.

Observable end-state: `clp .model set::opus` writes `claude-opus-4-8` to `settings.json`; `clp .model set::sonnet` writes `claude-sonnet-5`; the `imodel::opus` shorthand in `.usage` and `.account.use` resolves to `claude-opus-4-8`; the `imodel::sonnet` shorthand resolves to `claude-sonnet-5`; the auto-select path in `src/usage/subprocess.rs` yields `claude-sonnet-5` when sonnet conditions are met; `ISOLATED_DEFAULT_MODEL = "claude-opus-4-8"` and `REFRESH_DEFAULT_MODEL = "claude-sonnet-5"` in `claude_runner_core/src/isolated.rs`; existing tests FT-05 and FT-06 in `tests/cli/model_test.rs` pass with the updated assertions; `isolated_defaults_test.rs` and `invariant_trace_universality_test.rs` pass; `w3 .test level::3` passes with zero failures in `claude_profile`, `claude_runner_core`, and `claude_runner`.

## In Scope

- `src/usage/types.rs` — function `map_model_shorthand()` (around line 430): change `"claude-opus-4-6"` → `"claude-opus-4-8"` and `"claude-sonnet-4-6"` → `"claude-sonnet-5"` in the match arms for `"opus"` and `"sonnet"` shorthands
- `src/usage/subprocess.rs` — function `resolve_model()` (lines 35–36, 52): change all three `claude-sonnet-4-6` and `claude-opus-4-6` occurrences to `claude-sonnet-5` and `claude-opus-4-8` respectively; update doc comments on lines 18, 24, 25 to reflect the new IDs
- `src/registry.rs` — parameter description strings (lines 192, 227, 253, 268): update inline model ID examples in `with_description()` calls from `claude-opus-4-6` → `claude-opus-4-8` and `claude-sonnet-4-6` → `claude-sonnet-5`
- `src/usage/api_switch.rs` — doc comment (line 215): update `claude-opus-4-6` reference in the comment to `claude-opus-4-8`
- Existing test assertions in `tests/cli/model_test.rs` (FT-05, FT-06): update expected values if currently asserting the old stale IDs
- `module/claude_runner_core/src/isolated.rs` — `ISOLATED_DEFAULT_MODEL` constant (line 28): change `"claude-opus-4-6"` → `"claude-opus-4-8"`; `REFRESH_DEFAULT_MODEL` constant (line 50): change `"claude-sonnet-4-6"` → `"claude-sonnet-5"`; update doc comments on lines 26–28 and 47–50 to reflect current IDs
- `module/claude_runner/tests/isolated_defaults_test.rs` — assertions at lines 61 and 71: update expected strings from `"claude-opus-4-6"` → `"claude-opus-4-8"` and `"claude-sonnet-4-6"` → `"claude-sonnet-5"`
- `module/claude_runner/tests/invariant_trace_universality_test.rs` — assertion at line 135: update expected `--model claude-sonnet-4-6` → `--model claude-sonnet-5`
- `module/claude_runner/tests/docs/invariant/005_isolated_subprocess_defaults.md` — update documented constant values from stale IDs to current IDs

## Out of Scope

- Documentation files — stale IDs in `docs/` and `tests/docs/` were already bulk-replaced in the preceding documentation phase; no doc edits needed here
- `STATIC_MODELS` in `claude_quota` — populated by Task 007 with correct current IDs
- Adding new model shorthands or values — only ID string updates; no behavioral change

## Work Procedure

1. Read `src/usage/types.rs` around line 430 to locate `map_model_shorthand()` and confirm the exact match arm strings
2. Read `src/usage/subprocess.rs` lines 15–60 to locate `resolve_model()` and all `claude-sonnet-4-6` / `claude-opus-4-6` occurrences including doc comments
3. Read `src/registry.rs` lines 185–275 to locate all four `with_description()` calls containing the stale IDs
4. Read `src/usage/api_switch.rs` around line 215 to locate the doc comment
5. Apply fixes in `src/usage/types.rs`: change `"claude-opus-4-6"` → `"claude-opus-4-8"` and `"claude-sonnet-4-6"` → `"claude-sonnet-5"` in `map_model_shorthand()`
6. Apply fixes in `src/usage/subprocess.rs`: update all string literals and doc comments
7. Apply fixes in `src/registry.rs`: update all four `with_description()` strings
8. Apply fix in `src/usage/api_switch.rs`: update the doc comment
8b. Update test assertion strings in `tests/cli/model_test.rs`: search for all occurrences of `claude-opus-4-6` and `claude-sonnet-4-6` in the test file (FT-05 line 144, FT-06 line 161, test matrix header lines 20-21, and any additional fixture occurrences); replace with `claude-opus-4-8` and `claude-sonnet-5` respectively; run `grep -n "claude-opus-4-6\|claude-sonnet-4-6" tests/cli/model_test.rs` to confirm zero remaining
8c. In `module/claude_runner_core/src/isolated.rs`: change `ISOLATED_DEFAULT_MODEL = "claude-opus-4-6"` → `"claude-opus-4-8"` (line 28) and `REFRESH_DEFAULT_MODEL = "claude-sonnet-4-6"` → `"claude-sonnet-5"` (line 50); update surrounding doc comments to match
8d. In `module/claude_runner/tests/isolated_defaults_test.rs`: change assertion at line 61 from `"claude-opus-4-6"` → `"claude-opus-4-8"` and at line 71 from `"claude-sonnet-4-6"` → `"claude-sonnet-5"`
8e. In `module/claude_runner/tests/invariant_trace_universality_test.rs`: change assertion at line 135 from `--model claude-sonnet-4-6` → `--model claude-sonnet-5`
8f. In `module/claude_runner/tests/docs/invariant/005_isolated_subprocess_defaults.md`: update all stale ID values to current IDs
9. Run `grep -rn "claude-opus-4-6\|claude-sonnet-4-6" src/ module/claude_runner_core/src/ module/claude_runner/tests/` from `WORKSPACE_ROOT` to confirm zero remaining occurrences
10. Run `w3 .test level::3` in `module/claude_profile/`; then run `w3 .test level::3` in `module/claude_runner_core/` and `module/claude_runner/`; confirm FT-05 (`set::opus` → `claude-opus-4-8`) and FT-06 (`set::sonnet` → `claude-sonnet-5`) pass; zero failures across all three crates

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clp .model set::opus` | `map_model_shorthand()` | `settings.json` contains `"model":"claude-opus-4-8"`; exit 0 |
| `clp .model set::sonnet` | `map_model_shorthand()` | `settings.json` contains `"model":"claude-sonnet-5"`; exit 0 |
| `imodel::opus` on `.usage`/`.account.use` | `resolve_model()` | Subprocess launched with `--model claude-opus-4-8` |
| `imodel::sonnet` on `.usage`/`.account.use` | `resolve_model()` | Subprocess launched with `--model claude-sonnet-5` |
| Auto-select sonnet condition met (`son_idle=true`) | `resolve_model()` auto branch | Subprocess launched with `--model claude-sonnet-5` |
| `ISOLATED_DEFAULT_MODEL` in `claude_runner_core/src/isolated.rs` | Constant value | `"claude-opus-4-8"`; doc comment updated |
| `REFRESH_DEFAULT_MODEL` in `claude_runner_core/src/isolated.rs` | Constant value | `"claude-sonnet-5"`; doc comment updated |
| `isolated_defaults_test.rs` assertions (lines 61, 71) | Test assertions | Assert `"claude-opus-4-8"` and `"claude-sonnet-5"`; pass |
| `invariant_trace_universality_test.rs` assertion (line 135) | Test assertion | Asserts `--model claude-sonnet-5`; passes |
| `grep -rn "claude-opus-4-6\|claude-sonnet-4-6" src/ module/` | Exhaustive search | Zero matches across all source files and test files |
| `w3 .test level::3` (all three crates) | Full test suite | Zero failures; zero clippy warnings in `claude_profile`, `claude_runner_core`, `claude_runner` |

## Related Documentation

- `docs/feature/035_model_command.md` — specifies `set::opus` → `claude-opus-4-8`, `set::sonnet` → `claude-sonnet-5`
- `docs/feature/026_subprocess_model_effort.md` — specifies `imodel::opus` → `claude-opus-4-8`, `imodel::sonnet` → `claude-sonnet-5`
- `docs/algorithm/001_touch_model_selection.md` — `resolve_model()` decision table
- `docs/cli/param/035_imodel.md` — `imodel::` parameter values table
- `docs/cli/param/055_set.md` — `set::` parameter values table
- `tests/docs/feature/035_model_command.md` — FT-05, FT-06 assertions

## History

- **[2026-07-02]** `CREATED` — Fix stale model IDs `claude-opus-4-6` and `claude-sonnet-4-6` in `map_model_shorthand()`, `resolve_model()`, registry description strings, and doc comments.

## Verification Findings

**Round 1 — FAILED (MOST Goal Quality: M dimension; Implementation Readiness). Resolved before re-verify.**

Finding 1 (M): Goal section said "replace stale IDs" but did not state that the old IDs are rejected by the Claude API, making the consequence of inaction unclear.

Fix applied: Added motivation paragraph explaining that `claude-opus-4-6` / `claude-sonnet-4-6` are no longer accepted by the API, causing subprocess launch failures.

Finding 2 (IR): Work Procedure had no step to update test assertions in `tests/cli/model_test.rs`. FT-05 (line 144) and FT-06 (line 161) currently assert the stale IDs — applying source fixes without updating them would immediately cause test suite failure in Step 10.

Fix applied: Added Step 8b explicitly updating test assertion strings in `tests/cli/model_test.rs` and including a verification grep before proceeding to Step 9.

## Verification Record

**Round 2 — PASSED (all 4 dimensions). Date: 2026-07-02.**

| Dimension | Result | Agent |
|-----------|--------|-------|
| Scope Coherence | PASS | a0ea471221858d539 (Round 1) |
| MOST Goal Quality | PASS | ad9d53ce4f2f64a50 (M re-verify Round 2) |
| Value / YAGNI | PASS | a5ce077ac5d335414 (Round 1) |
| Implementation Readiness | PASS | a964796c19cf303e9 (IR re-verify Round 2) |
