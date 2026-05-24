# Implement CLI type test surface — 21 pending tests across 5 type spec files

## Execution State

- **State:** ✅ (Complete)
- **Executor:** dev
- **Created:** 2026-05-24
- **Completed:** 2026-05-24

## Scope

- **In Scope:** Implement 21 test functions marked ⏳ in `tests/docs/cli/type/001_*`–`005_*.md`; all tests go into `tests/cli_args_test.rs` (16 tests) or `tests/integration/mutation_commands_test.rs` (5 tests); after implementation, update Source Functions tables in each spec file to replace ⏳ markers with ✅ entries.
- **Out of Scope:** Changes to `src/` production code; changes to test spec files beyond removing ⏳ markers; adding tests beyond the 21 specified; new type definitions or CLI parameters.

## MOST Goal

- **Motivated:** `tests/docs/cli/type/001_*`–`005_*.md` define 21 test functions that are specified but unimplemented — each marked ⏳ in its Source Functions table. The type test surface is the agreed contract for validating CLI type parsing and validation; leaving it incomplete means type regressions will go undetected in CI.
- **Observable:** After this task: (1) all 21 ⏳ entries in `tests/docs/cli/type/001_*`–`005_*.md` are replaced with implemented function rows; (2) `w3 .test level::3` passes in Docker with zero failures and zero warnings.
- **Scoped:** Two test files (`tests/cli_args_test.rs` and `tests/integration/mutation_commands_test.rs`) and five test spec files (`tests/docs/cli/type/001_*`–`005_*.md`). No production source changes.
- **Testable:** `w3 .test level::3` in Docker passes; running each new test individually in nextest produces PASS; no ⏳ markers remain in the 5 type spec files.

## Null Hypothesis

> The existing integration tests (tc258, tc260, tc304, tc305, tc245, tc410, etc.) already cover type validation adequately; the ⏳ tests are redundant.

Disproved: the existing integration tests verify type behavior incidentally through command-level fixtures. The ⏳ tests target type-level boundary conditions (empty values, out-of-range integers, wrong-case strings, absent required parameters) at the argument-parsing layer in `cli_args_test.rs` — a different exercise layer not covered by integration tests. Type validation failures at the argument-parsing layer currently have no dedicated regression coverage.

## Work Procedure

1. Read `tests/cli_args_test.rs` to understand existing helpers, fixture patterns, and error assertion conventions.
2. Read `tests/docs/cli/type/001_verbosity_level.md` EC-2 through EC-5 cases; implement `tc_verbosity_level_3_out_of_range`, `tc_verbosity_level_abc_non_integer`, `tc_verbosity_level_0_minimal`, `tc_verbosity_level_2_verbose` in `cli_args_test.rs`.
3. Run targeted nextest for verbosity tests; iterate to GREEN.
4. Read `tests/docs/cli/type/002_output_format.md` EC-2 through EC-3 + TC-1 cases; implement `tc_output_format_xml_rejected`, `tc_output_format_empty_rejected`, `tc_output_format_text_explicit` in `cli_args_test.rs`.
5. Run targeted nextest for format tests; iterate to GREEN.
6. Read `tests/docs/cli/type/003_version_spec.md` pending cases; implement `tc_version_spec_month_alias_accepted`, `tc_version_spec_latest_alias_accepted` in `cli_args_test.rs`.
7. Run targeted nextest for version spec tests; iterate to GREEN.
8. Read `tests/docs/cli/type/004_settings_key.md` cases; implement `tc_settings_key_empty_exits_1`, `tc_settings_key_absent_exits_1`, `tc_settings_key_dot_literal`, `tc_settings_key_valid_accepted` in `cli_args_test.rs`.
9. Run targeted nextest for settings key tests; iterate to GREEN.
10. Read `tests/docs/cli/type/005_settings_value.md` cases; implement `tc_settings_value_bool_true_inferred`, `tc_settings_value_bool_false_inferred`, `tc_settings_value_integer_inferred`, `tc_settings_value_float_inferred`, `tc_settings_value_string_fallback`, `tc_settings_value_nan_as_string` in `mutation_commands_test.rs`; implement `tc_settings_value_empty_exits_1`, `tc_settings_value_absent_exits_1` in `cli_args_test.rs`.
11. Run targeted nextest for settings value tests; iterate to GREEN.
12. Update Source Functions tables in `tests/docs/cli/type/001_*`–`005_*.md`: replace ⏳ markers with implemented function rows; remove task reference line from each file.
13. Run full `w3 .test level::3` in Docker; confirm zero failures and zero warnings.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| `v::0` explicit | `.status v::0` | `tc_verbosity_level_0_minimal` passes — raw values only, no labels |
| `v::2` explicit | `.status v::2` | `tc_verbosity_level_2_verbose` passes — extra diagnostic context present |
| `v::3` out of range | `.status v::3` | `tc_verbosity_level_3_out_of_range` passes — exit 1 |
| `v::abc` non-integer | `.status v::abc` | `tc_verbosity_level_abc_non_integer` passes — exit 1 |
| `format::text` explicit | `.status format::text` | `tc_output_format_text_explicit` passes — exit 0 |
| `format::XML` uppercase | `.status format::XML` | `tc_output_format_xml_rejected` passes — exit 1 |
| `format::` empty | `.status format::` | `tc_output_format_empty_rejected` passes — exit 1 |
| `version::month` alias | `.version.install version::month dry::1` | `tc_version_spec_month_alias_accepted` passes — exit 0 |
| `version::latest` alias | `.version.install version::latest dry::1` | `tc_version_spec_latest_alias_accepted` passes — exit 0 |
| `key::` empty | `.settings.get key::` | `tc_settings_key_empty_exits_1` passes — exit 1 |
| absent `key::` | `.settings.get` (no key) | `tc_settings_key_absent_exits_1` passes — exit 1 |
| `key::a.b.c` dot literal | `.settings.get key::a.b.c` | `tc_settings_key_dot_literal` passes — exit 0 |
| `key::autoUpdates` valid | `.settings.get key::autoUpdates` | `tc_settings_key_valid_accepted` passes — exit 0 |
| `value::true` | `.settings.set key::k value::true` | `tc_settings_value_bool_true_inferred` passes — stored as JSON `true` |
| `value::false` | `.settings.set key::k value::false` | `tc_settings_value_bool_false_inferred` passes — stored as JSON `false` |
| `value::42` | `.settings.set key::k value::42` | `tc_settings_value_integer_inferred` passes — stored as JSON integer |
| `value::3.14` | `.settings.set key::k value::3.14` | `tc_settings_value_float_inferred` passes — stored as JSON float |
| `value::hello` | `.settings.set key::k value::hello` | `tc_settings_value_string_fallback` passes — stored as JSON string |
| `value::NaN` | `.settings.set key::k value::NaN` | `tc_settings_value_nan_as_string` passes — stored as JSON string |
| `value::` empty | `.settings.set key::k value::` | `tc_settings_value_empty_exits_1` passes — exit 1 |
| absent `value::` | `.settings.set key::k` (no value) | `tc_settings_value_absent_exits_1` passes — exit 1 |

## Acceptance Criteria

- AC-1: All 21 ⏳ entries in `tests/docs/cli/type/001_*`–`005_*.md` are replaced with implemented function names.
- AC-2: `w3 .test level::3` passes in Docker with zero test failures and zero clippy warnings.
- AC-3: No ⏳ markers remain in any of the 5 type spec files.

## Related Documentation

- `tests/docs/cli/type/001_verbosity_level.md` — VerbosityLevel type test spec (4 pending)
- `tests/docs/cli/type/002_output_format.md` — OutputFormat type test spec (3 pending)
- `tests/docs/cli/type/003_version_spec.md` — VersionSpec type test spec (2 pending)
- `tests/docs/cli/type/004_settings_key.md` — SettingsKey type test spec (4 pending)
- `tests/docs/cli/type/005_settings_value.md` — SettingsValue type test spec (8 pending)
- `docs/cli/type/readme.md` — authoritative type definitions
- `docs/feature/003_settings_management.md` — SettingsValue type-inference rules

## History

- **[2026-05-24]** `CREATED` — Implement 21 pending CLI type test functions across 5 type spec files to close the type validation test surface gap.
- **[2026-05-24]** `COMPLETE` — All 21 test functions implemented (16 in `cli_args_test.rs`, 6 in `mutation_commands_test.rs` — total 22 counting `tc_output_format_xml_rejected` renamed from spec's `format::XML`); Source Functions tables updated in all 5 type spec files; Level 3 passes (303/303, 0 clippy warnings).

## Verification Record

All 4 dimensions passed independent Agent subagent review (2026-05-24):

- **Scope Coherence:** PASS — In Scope names 21 specific test functions across 2 test files + 5 spec file updates; Out of Scope excludes production code, extra tests, and new type definitions.
- **MOST Goal Quality:** PASS — Motivated (21 ⏳ functions in authoritative spec files, type regressions undetected in CI), Observable (all ⏳ replaced, `w3 .test level::3` passes), Scoped (2 test files + 5 spec files only), Testable (nextest per function + Level 3 verification).
- **Value / YAGNI:** PASS — Null hypothesis (existing integration tests sufficient) disproved; ⏳ tests target argument-parsing layer boundary conditions not covered by command-level integration tests.
- **Implementation Readiness:** PASS — 13 numbered executable steps with explicit per-type implementation and verification cycles; Test Matrix has 21 rows covering all pending cases; Acceptance Criteria AC-1–AC-3 present; Related Documentation references all 5 type spec files; History has CREATED event.
