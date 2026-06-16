# Task 004 -- Test Surface Remediation

## Execution State

- **State:** ❓ (Unverified)
- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **Priority:** 3
- **Value:** 7
- **Easiness:** 5
- **Safety:** 9
- **Advisability:** 315
- **Dir:** .
- **Validated By:** null
- **Validation Date:** null
- **Blocked Reason:** null
- **Closes:** null

## MOST Goal

- **Motivated:** The test surface audit identified 29 problems across `tests/docs/` spec files: 12 test spec files with pending implementation (⏳ status -- 2 type specs, 5 user story specs, and the config-related param/param_group/command specs all lack implemented test functions), 46 cases using wrong TC-NNN prefix instead of the element-type prefix (IT-/EC-/CC-), 1 command spec (`013_config.md`) with zero GWT case detail sections, and 6 algorithm cases using AT- instead of AC-. Without remediation, the Coverage Gate (checks 101-112) remains BLOCKED and the test surface contract between documentation and code is unenforceable.
- **Observable:** After this task: (1) all ⏳ entries in `tests/docs/cli/type/006_*.md`, `007_*.md`, all 5 `user_story/*.md`, and all pending entries across param/param_group/command specs are replaced with implemented function names; (2) all TC-NNN prefixed cases in command specs are renumbered to IT-N; (3) all TC-NNN prefixed cases in param specs are renumbered to EC-N; (4) `013_config.md` has full GWT case detail sections for all 17 cases; (5) `002_config_resolution.md` AT- prefix corrected to AC-; (6) `verb/test` passes with zero failures.
- **Scoped:** Changes confined to `tests/` (Rust test files) and `tests/docs/` (spec files). No production source changes in `src/`. No changes to `docs/` (documentation consistency already established).
- **Testable:** `verb/test` passes in container. Zero ⏳ markers remain in any spec file under `tests/docs/`. `grep -r 'TC-[0-9]' tests/docs/cli/command/` returns zero matches. `grep -r 'AT-' tests/docs/algorithm/` returns zero matches.

## Null Hypothesis

> The existing integration tests already cover the behaviors specified in the ⏳ spec files (type 006/007, user stories 001-005); the spec-internal quality issues (TC-NNN prefix, missing GWT) are cosmetic and do not affect test coverage.

Disproved: (1) ConfigScope and ConfigKey type specs define boundary conditions (case sensitivity, empty values, unknown variants) that have no dedicated test functions -- these are type-level validation tests at the argument-parsing layer, distinct from command-level integration tests. (2) User story specs define end-to-end workflow acceptance criteria that integration tests exercise incidentally per-command but never validate as a complete workflow sequence. (3) The TC-NNN prefix contamination prevents automated tooling from distinguishing command test cases from type test cases, breaking the Coverage Gate check 103 (prefix match). (4) Missing GWT sections in 013_config.md means 17 test cases have no documented Given/When/Then contract -- making the spec unreviewable and the implementation untraceable.

## In Scope

### Implementation Work

- Implement test functions for `tests/docs/cli/type/006_config_scope.md` (6 TC- cases) in `tests/cli_args_test.rs`
- Implement test functions for `tests/docs/cli/type/007_config_key.md` (6 TC- cases) in `tests/cli_args_test.rs` or `tests/integration/config_commands_test.rs`
- Implement test functions for `tests/docs/cli/user_story/001_environment_check.md` through `005_version_pinning.md` (28 US- cases total) in `tests/integration/`
- Update Source Functions tables in all implemented spec files to replace ⏳ with function names

### Spec Quality Fixes

- `tests/docs/cli/command/013_config.md`: add full `### IT-N:` GWT case detail sections for all 17 cases
- `tests/docs/cli/command/004_version_install.md`: renumber 17 TC-NNN cases to IT-N continuation
- `tests/docs/cli/command/005_version_guard.md`: renumber TC-418 to IT-N
- `tests/docs/cli/command/006_version_list.md`: renumber 7 TC-NNN cases to IT-N
- `tests/docs/cli/command/009_settings_show.md`: renumber 4 TC-NNN cases to IT-N
- `tests/docs/cli/command/010_settings_get.md`: renumber 4 TC-NNN cases to IT-N
- `tests/docs/cli/command/011_settings_set.md`: renumber 13 TC-NNN cases to IT-N
- `tests/docs/cli/param/` specs: renumber TC-NNN cases to EC-N in 8 param files
- `tests/docs/algorithm/002_config_resolution.md`: renumber AT-01 through AT-06 to AC-1 through AC-6
- Fix bare bold `**Note:**` / `**Expected:**` labels to use `- ` dash prefix in affected command and param specs

## Out of Scope

- Production source code changes in `src/`
- Changes to `docs/` (documentation consistency already established)
- Creating new spec files (all spec files already exist after doc_tsk Step 2)
- Changes to `claude_version_core` crate (cross-package, not caused by interface change)

## Work Procedure

### Phase 1 -- Spec Quality Fixes (documentation)

1. Read `tests/docs/cli/command/013_config.md`; add full `### IT-N:` GWT case detail sections for all 17 cases (Given/When/Then/Exit format matching sibling command specs).

2. For each command spec with TC-NNN cases (`004_version_install.md`, `005_version_guard.md`, `006_version_list.md`, `009_settings_show.md`, `010_settings_get.md`, `011_settings_set.md`): renumber TC-NNN to sequential IT-N continuation after existing IT-N cases. Update Test Case Index table to match.

3. For each param spec with TC-NNN cases (`001_version.md`, `002_dry.md`, `003_force.md`, `004_verbosity.md`, `005_format.md`, `006_key.md`, `007_value.md`, `009_count.md`): renumber TC-NNN to sequential EC-N continuation. Update Test Case Index and divergence pair references.

4. In `tests/docs/algorithm/002_config_resolution.md`: rename AT-01 through AT-06 to AC-1 through AC-6. Update readme.md Overview Table entry ("AT- test cases" -> "AC- test cases").

5. Fix bare bold labels: search all spec files for consecutive `**Note:**`, `**Expected:**`, `**Isolation:**` without `- ` prefix; add dash prefix.

### Phase 2 -- Type Test Implementation

6. Read `tests/cli_args_test.rs` to understand existing helpers and fixtures.

7. Implement 6 test functions for `006_config_scope.md` (TC-1 through TC-6): scope enum validation at argument-parsing layer.

8. Implement 6 test functions for `007_config_key.md` (TC-1 through TC-6): config key validation at argument-parsing or integration layer.

9. Run targeted nextest for new type tests; iterate to GREEN.

10. Update Source Functions tables in `006_config_scope.md` and `007_config_key.md`.

### Phase 3 -- User Story Test Implementation

11. Implement 4 test functions for `001_environment_check.md` (US-1 through US-4) in `tests/integration/`.

12. Implement 6 test functions for `002_version_upgrade.md` (US-1 through US-6).

13. Implement 6 test functions for `003_process_lifecycle.md` (US-1 through US-6).

14. Implement 6 test functions for `004_settings_management.md` (US-1 through US-6).

15. Implement 6 test functions for `005_version_pinning.md` (US-1 through US-6).

16. Update Source Functions tables in all 5 user story spec files.

### Phase 4 -- Verification

17. Run `verb/test` in container. All tests pass, zero warnings.

18. Verify zero ⏳ markers remain: `grep -r '⏳' tests/docs/`.

19. Verify zero TC-NNN in command specs: `grep -r 'TC-[0-9]' tests/docs/cli/command/`.

20. Verify zero AT- in algorithm specs: `grep -r 'AT-' tests/docs/algorithm/`.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `scope::user` on `.config` set | ConfigScope TC-1 | exit 0; writes to user config |
| `scope::project` on `.config` set | ConfigScope TC-2 | exit 0; writes to project config |
| `scope::global` (unknown variant) | ConfigScope TC-4 | exit 1; invalid scope |
| `scope::USER` (wrong case) | ConfigScope TC-5 | exit 1; case-sensitive enum |
| `key::model` on `.config` get | ConfigKey TC-1 | exit 0; resolves catalog default |
| `key::myCustomSetting` on `.config` get | ConfigKey TC-2 | exit 0; arbitrary key, no default |
| `key::` (empty) on `.config` | ConfigKey TC-6 | exit 1; empty key rejected |
| `cm .status` (environment check) | US-001 US-1 | exit 0; version + session count + account |
| `cm .status format::json` | US-001 US-2 | exit 0; valid JSON with same fields |
| `cm .version.install version::Y dry::1` | US-002 US-1 | exit 0; preview shown, no install |
| `cm .version.install version::Y` | US-002 US-2 | exit 0; installed + version lock |
| `cm .processes` | US-003 US-1 | exit 0; PID list |
| `cm .processes.kill force::1` | US-003 US-5 | exit 0; SIGKILL directly |
| `cm .settings.show` | US-004 US-1 | exit 0; all key-value pairs |
| `cm .settings.set key::X value::true` | US-004 US-6 | exit 0; stored as JSON bool |
| `cm .version.list` | US-005 US-1 | exit 0; aliases with resolved versions |

## Acceptance Criteria

- AC-1: All ⏳ entries in `tests/docs/cli/type/006_*.md` and `007_*.md` replaced with implemented function names.
- AC-2: All ⏳ entries in `tests/docs/cli/user_story/001_*.md` through `005_*.md` replaced with implemented function names.
- AC-3: Zero TC-NNN prefixed cases remain in any `tests/docs/cli/command/*.md` file.
- AC-4: Zero TC-NNN prefixed cases remain in any `tests/docs/cli/param/*.md` file.
- AC-5: All 17 cases in `tests/docs/cli/command/013_config.md` have full GWT detail sections.
- AC-6: `tests/docs/algorithm/002_config_resolution.md` uses AC- prefix (not AT-).
- AC-7: `verb/test` passes in container with zero failures and zero warnings.

## Related Documentation

- `tests/docs/cli/type/006_config_scope.md` -- ConfigScope type test spec (6 cases, ⏳)
- `tests/docs/cli/type/007_config_key.md` -- ConfigKey type test spec (6 cases, ⏳)
- `tests/docs/cli/user_story/001_environment_check.md` -- Environment Check user story test spec (4 cases, ⏳)
- `tests/docs/cli/user_story/002_version_upgrade.md` -- Version Upgrade user story test spec (6 cases, ⏳)
- `tests/docs/cli/user_story/003_process_lifecycle.md` -- Process Lifecycle user story test spec (6 cases, ⏳)
- `tests/docs/cli/user_story/004_settings_management.md` -- Settings Management user story test spec (6 cases, ⏳)
- `tests/docs/cli/user_story/005_version_pinning.md` -- Version Pinning user story test spec (6 cases, ⏳)
- `tests/docs/cli/command/013_config.md` -- Config command integration test spec (17 cases, missing GWT)
- `tests/docs/algorithm/002_config_resolution.md` -- Config resolution algorithm test spec (AT- prefix violation)
- `docs/cli/type/06_config_scope.md` -- ConfigScope type definition (authoritative source)
- `docs/cli/type/07_config_key.md` -- ConfigKey type definition (authoritative source)
- `docs/cli/user_story/` -- All 5 user story definitions (authoritative source)
- Related: 003 (`task/claude_version/003_config_command.md` -- .config command implementation, related scope)

## History

- **[2026-06-16]** `CREATED` -- Remediate test surface gaps: implement pending type/user-story tests, fix prefix contamination and missing GWT sections across 29 spec file problems.
