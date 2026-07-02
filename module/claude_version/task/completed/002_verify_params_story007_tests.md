# Task 002: Verify `.params` Command and Story 007 Tests in Container

## Execution State

- **State:** ✅ (Closed)
- **Executor:** any

## MOST Goal

- **Motivated:** 24 test functions were added across two files — `params_command_test.rs` (IT-1 through IT-14) and `user_story_test.rs` (US-1 through US-10 for story 007) — implementing the new `.params` command and the params inspection user story. These were added as part of the same normalization session that introduced `kind::`, `ParamKind`, `.params`, and user story 007 to the doc collection. No container test run has been executed to validate these 24 functions against the live binary.
- **Observable:** `./verb/test` exits 0; all 24 named functions appear in the passing output; no regressions in the rest of the test suite.
- **Scoped:** `tests/cli/params_command_test.rs` (IT-1–IT-14) and `tests/cli/user_story_test.rs` (US-1–US-10 for story 007) in the `claude_version` crate.
- **Testable:** `./verb/test` exits 0 with all 24 function names listed in the PASSED section of test output.

## In Scope

- Run the full test suite via `./verb/test` in the container
- Diagnose and fix any failures among the 24 functions
- Ensure no regressions in the existing test suite

## Out of Scope

- `kind_param_test.rs` EC-1–EC-7 and TC-1–TC-6 (covered by task 001)
- `format_surface_test.rs` FM-5 and `format_param_test.rs` EC-20 (covered by task 001)
- Adding test spec cases beyond those already defined in `tests/docs/`
- Source code behavior changes (binary bugs found during this run → new bug report)

## Null Hypothesis

The 24 functions cover behavior already exercised by existing tests; no additional container run is needed.

**Disproof:** The IT- and US- functions cover distinct behavioral paths not previously tested:
- IT-1 through IT-14: end-to-end `.params` command behavior — show-all mode, single-param mode, kind filter, format selection, verbosity levels, env var reads, config reads, CLI-only annotation, alphabetical sort, and all three error exits (0 / 1 / 2)
- US-1 through US-10: acceptance criteria for the params inspection user story (persona-level workflow: discovery, deep-dive, filtering, env annotation, JSON output, error handling, ordering)

## Work Procedure

1. Run `./verb/test` in the container
2. Identify any failures among the 24 functions by name
3. For each failure, diagnose root cause: wrong assertion, unexpected binary output, or environment assumption
4. Fix the failing function (adjust assertions to match spec-defined behavior)
5. Re-run until all 24 new functions pass and no regressions appear
6. Document any unexpected binary behavior found as a separate bug report

## Test Matrix

| Input Scenario | Function | Expected |
|---------------|----------|----------|
| `.params` (no args) | `it01_params_show_all_min_entries` | ≥35 top-level entries; each annotated; exit 0 |
| `.params key::model` (no env/config) | `it02_params_single_model_full_detail` | shows `--model`, `CLAUDE_MODEL`, `config model`, default; exit 0 |
| `.params kind::config` | `it03_params_kind_config_filters` | config-key params present; env-only absent; exit 0 |
| `.params kind::env` | `it04_params_kind_env_filters` | env-var params present; config-only absent; exit 0 |
| `.params key::model` + CLAUDE_MODEL set | `it05_params_env_override_visible` | shows env value with `(env)` annotation; exit 0 |
| `.params key::bash_timeout` | `it06_params_env_only_param_unset` | env-only param; shows unset + default 120000; exit 0 |
| `.params format::json` | `it07_params_json_output_structure` | stdout starts with `[`; each entry has `"name"` key; exit 0 |
| `.params key::print` | `it08_params_cli_only_annotation` | shows `--print`; CLI-only annotation present; exit 0 |
| `.params v::0` | `it09_params_compact_v0_output` | values only; no "Forms:" labels; exit 0 |
| `.params key::model` (no env, no config) | `it10_params_default_annotation` | shows `(default)` annotation; exit 0 |
| `.params` (show-all) | `it11_params_show_all_alphabetical` | param names in ascending alphabetical order; exit 0 |
| `.params key::NONEXISTENT_KEY` | `it12_params_unknown_key_exits_2` | exit 2 |
| `.params kind::badvalue` | `it13_params_invalid_kind_exits_1` | exit 1 |
| `.params format::xml` | `it14_params_invalid_format_exits_1` | exit 1 |
| `.params` (no args) | `us01_007_params_show_all_entries` | ≥35 params; each has source annotation or CLI-only marker; exit 0 |
| `.params key::model` (no env/config) | `us02_007_params_single_model_forms` | `--model`, `CLAUDE_MODEL`, `config model`, default `claude-sonnet-5`, `(default)`; exit 0 |
| `.params kind::config` | `us03_007_params_kind_config_only` | `model`, `theme` present; `bash_timeout` absent; exit 0 |
| `.params kind::env` | `us04_007_params_kind_env_only` | `bash_timeout` present; `theme` absent; exit 0 |
| `.params key::model` + env override | `us05_007_params_env_override_annotated` | env value with `(env)` annotation; exit 0 |
| `.params key::print` | `us06_007_params_cli_only_print` | `-p / --print` form; CLI-only annotation; exit 0 |
| `.params format::json` | `us07_007_params_json_array_output` | valid JSON array; each element has `name` key; exit 0 |
| `.params key::UNKNOWN` | `us08_007_params_unknown_key_exits_2` | exit 2 |
| `.params kind::bad` | `us09_007_params_invalid_kind_exits_1` | exit 1 |
| `.params` (show-all) | `us10_007_params_show_all_alphabetical` | param names in ascending alphabetical order; exit 0 |

## Validation

- **Pass criterion:** `./verb/test` exits 0; all 24 function names appear in the PASSED section
- **Fail criterion:** Any of the 24 functions appears in FAILED; or any existing test regresses

## Related Documentation

- `docs/cli/command/params.md` — `.params` command specification
- `docs/cli/param/13_kind.md` — `kind::` parameter specification
- `docs/cli/type/08_param_kind.md` — `ParamKind` type specification
- `docs/cli/user_story/007_params_inspection.md` — user story 007 specification
- `tests/docs/cli/command/14_params.md` — IT-1 through IT-14 test specs
- `tests/docs/cli/user_story/07_params_inspection.md` — US-1 through US-10 specs
- `tests/cli/params_command_test.rs` — IT-1–IT-14 implementation file
- `tests/cli/user_story_test.rs` — US-1–US-10 implementation file (story 007 section, lines 655–820+)

**Closes:** null

## History

- **[2026-07-02]** `CREATED` — Verify 24 .params and story 007 test functions (IT-1–IT-14, US-1–US-10) pass in container.
- **[2026-07-02]** `VERIFIED` — MAAV gate passed: 4/4 effective agents across 2 rounds (Scope Coherence, MOST Goal Quality, Value/YAGNI ×2, Implementation Readiness ×2); one prior-session false positive refuted by direct filesystem evidence.
- **[2026-07-02]** `CLOSED` — `verb/test` exited 0: 582/582 tests passed, 0 failed, 0 skipped. All 24 named functions (IT-1–IT-14, US-1–US-10) confirmed in PASSED output. No regressions. Clippy clean.

## Verification Record

- **Date:** 2026-07-02
- **Method:** MAAV — 6 independent subagents across two dispatch rounds (prior session + re-dispatch after context compaction)
- **Scope Coherence:** PASS — In Scope concrete, Out of Scope non-empty and meaningful, observable outcome binary-verifiable, all 8 Related Documentation paths confirmed present
- **MOST Goal Quality:** PASS — all 4 dimensions pass; Motivated states unexecuted concrete need, Observable is binary and mechanical, Scoped to two specific files, Testable is unambiguous
- **Value / YAGNI:** PASS — both independent adversarial agents confirmed: both test files are untracked (no prior container run), 24 functions cover distinct behavioral paths with no coverage in task 001, no YAGNI violation, zero overlap
- **Implementation Readiness:** PASS — Work Procedure executable (`verb/test` confirmed present), all 24 function names verified in source files (IT-1–IT-14 at lines 33–299; US-1–US-10 at lines 655–808), all 8 Related Documentation paths confirmed present, Test Matrix has 24 rows with binary-verifiable outcomes. (One prior-session agent returned a false positive on path resolution; refuted by direct filesystem verification and independent adversarial agent confirmation.)
