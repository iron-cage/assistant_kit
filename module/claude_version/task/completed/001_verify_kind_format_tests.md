# Task 001: Verify kind_param and Format Tests in Container

## Execution State

- **State:** ✅ (Closed)
- **Executor:** any

## MOST Goal

- **Motivated:** 15 test functions were added across three files — `kind_param_test.rs` (EC-1–EC-7, TC-1–TC-6), `format_surface_test.rs` (FM-5), and `format_param_test.rs` (EC-20) — but the container test run has not been executed to validate them against the live binary.
- **Observable:** `./verb/test` exits 0; all 15 named functions appear in the passing output; no regressions in the existing test suite.
- **Scoped:** `tests/cli/` directory of the `claude_version` crate.
- **Testable:** `./verb/test` exits 0 with all 15 new functions listed as passed.

## In Scope

- Run the full test suite via `./verb/test` in the container
- Diagnose and fix any failures among the 15 new functions
- Ensure no regressions in the existing test suite

## Out of Scope

- Adding test spec cases beyond those already defined in `tests/docs/`
- Source code behavior changes (binary bugs found during this run → new bug report)

## Null Hypothesis

The new test functions cover behavior already exercised by existing tests; no additional container run is needed.

**Disproof:** The 15 functions cover distinct behavioral paths not previously tested:
- EC-1–EC-2 / TC-1–TC-2: `kind::config` vs `kind::env` filtering produces different output sets
- EC-4–EC-6 / TC-4–TC-6: distinct invalid inputs with different error causes (unknown, empty, wrong case)
- EC-7: `key::` supersede of `kind::` is a unique interaction path
- FM-5: verifies stderr is empty — not asserted by any existing FM test
- EC-20: `.params format::json` (different command from existing json format tests)

## Work Procedure

1. Run `./verb/test` in the container
2. Identify any failures among the 15 new functions by name
3. For each failure, diagnose root cause: wrong assertion, unexpected binary output, or environment assumption
4. Fix the failing function (adjust assertions to match spec-defined behavior)
5. Re-run until all 15 new functions pass and no regressions appear
6. Document any unexpected binary behavior found as a separate bug report

## Test Matrix

| Input Scenario | Function | Expected |
|---------------|----------|----------|
| `.params kind::config` | `kind_ec1_config_shows_config_params` | contains `model`; excludes `bash_timeout`; exit 0 |
| `.params kind::env` | `kind_ec2_env_shows_env_params` | contains `bash_timeout`; excludes `theme`; exit 0 |
| `.params` (no `kind::`) | `kind_ec3_absent_shows_all_params` | ≥35 top-level entries; exit 0 |
| `.params kind::invalid` | `kind_ec4_invalid_exits_1` | exit 1; stderr mentions `config` or `env` |
| `.params kind::` | `kind_ec5_empty_exits_1` | exit 1; stderr non-empty |
| `.params kind::CONFIG` | `kind_ec6_uppercase_exits_1` | exit 1 |
| `.params key::model kind::env` | `kind_ec7_ignored_when_key_present` | contains `--model` and `CLAUDE_MODEL`; exit 0 |
| `.params kind::config` | `kind_tc1_config_shows_config_params_only` | contains `model`; excludes `bash_timeout`; exit 0 |
| `.params kind::env` | `kind_tc2_env_shows_env_params_only` | contains `bash_timeout`; excludes `theme`; exit 0 |
| `.params` (no `kind::`) | `kind_tc3_absent_shows_all_params` | contains both `model` and `bash_timeout`; exit 0 |
| `.params kind::Config` | `kind_tc4_mixed_case_exits_1` | exit 1; stderr non-empty |
| `.params kind::all` | `kind_tc5_unknown_variant_exits_1` | exit 1; stderr mentions `config` or `env` |
| `.params kind::` | `kind_tc6_empty_exits_1` | exit 1; stderr non-empty |
| `.status format::json` | `fm05_02_json_stdout_only` | exit 0; stdout starts with `{`; stderr empty |
| `.params format::json` | `format_ec20_params_format_json_array` | exit 0; stdout starts with `[`; contains `"name"` |

## Validation

- **Pass criterion:** `./verb/test` exits 0; all 15 new function names appear in the PASSED section of test output
- **Fail criterion:** Any of the 15 functions appears in FAILED; or existing tests regress

## Related Documentation

- `tests/docs/cli/param/13_kind.md` — `kind::` parameter edge case spec (EC-1–EC-7)
- `tests/docs/cli/type/08_param_kind.md` — `ParamKind` type validation spec (TC-1–TC-6)
- `tests/docs/cli/format/02_json.md` — FM-5 JSON stdout-only spec
- `tests/docs/cli/param/05_format.md` — EC-20 `.params format::json` spec
- `tests/cli/kind_param_test.rs` — EC-1–EC-7 and TC-1–TC-6 implementing file
- `tests/cli/format_surface_test.rs` — FM-5 implementing file
- `tests/cli/format_param_test.rs` — EC-20 implementing file

**Closes:** null

## History

- **[2026-07-02]** `CREATED` — Verify 15 new test functions (EC-1–7, TC-1–6, FM-5, EC-20) pass in container.
- **[2026-07-02]** `VERIFIED` — MAAV gate passed: 4/4 independent agents (Scope Coherence, MOST Quality, Value/YAGNI, Implementation Readiness) returned PASS.
- **[2026-07-02]** `CLOSED` — `verb/test` exited 0: 582/582 tests passed, 0 failed, 0 skipped. All 15 named functions (EC-1–EC-7, TC-1–TC-6, FM-5, EC-20) confirmed in PASSED output. No regressions. Clippy clean.

## Verification Record

- **Date:** 2026-07-02
- **Method:** MAAV — 4 independent subagents dispatched in parallel
- **Scope Coherence:** PASS — In Scope concrete, Out of Scope meaningful, observable outcome binary-verifiable, no contradictions, single coherent domain
- **MOST Goal Quality:** PASS — all 4 dimensions pass; two minor documentation gaps noted (implicit full-suite assumption in Observable; no fix-quality constraint in Testable) that do not invalidate any dimension
- **Value / YAGNI:** PASS — adversarial agent confirmed via git status that `kind_param_test.rs` is untracked and both format files are modified post-commit; 15 functions cover distinct behavioral paths with no existing coverage
- **Implementation Readiness:** PASS — Work Procedure executable, all 15 matrix function names verified to exist in source files, all 6 Related Documentation paths exist, validation criterion mechanical
