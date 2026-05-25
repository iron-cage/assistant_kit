# User Story Acceptance Test Implementation

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** null
- **Validation Date:** null

## Goal

The `tests/docs/cli/user_story/` entity (created 2026-05-25) defines 25 RWS-N acceptance test
cases across 5 user story spec files. None of these scenarios have a corresponding Rust
implementation. The gap violates the test-mirror contract established for `tests/docs/cli/`:
every test spec file must have a Rust counterpart, as exists for `feature_cli_tool_test.rs`
(FT-N) and `operation_migration_guide_test.rs` (OP-N).

Implement all 25 acceptance tests as Rust integration tests in `tests/`, one file per user
story, mirroring the structure of existing `*_test.rs` files. Each test must exercise the
real CLI binary against an isolated temp fixture via `common::clg_cmd()`.

Task is complete when ALL of the following hold:
1. 5 new test files exist: `tests/us1_audit_session_history_test.rs` through
   `tests/us5_resume_claude_session_test.rs`
2. All 25 Rust test functions compile and pass: `cargo nextest run --test us1_*` through
   `tests/us5_*` exit 0 (25/25 passing)
3. Each test function's doc comment references its `RWS-N` source ID and the spec file
4. Each test uses an isolated `TempDir` fixture — no shared mutable state between tests
5. `RUSTFLAGS="-D warnings" cargo nextest run --all-features` exits 0 (no new warnings)

## In Scope

**US-1 (5 tests) — `tests/us1_audit_session_history_test.rs`:**
- `rws_1_status_shows_project_and_session_totals` — `.status` stdout contains project count and session count
- `rws_2_verbosity_2_shows_per_project_breakdown` — `.status verbosity::2` stdout contains per-project counts
- `rws_3_verbosity_0_outputs_machine_readable_format` — `.status verbosity::0` stdout matches `projects: N, sessions: N`
- `rws_4_count_sessions_returns_bare_integer` — `.count target::sessions` stdout is a single integer
- `rws_5_path_override_inspects_alternate_storage_root` — `.status path::{alt}` reflects alternate fixture

**US-2 (5 tests) — `tests/us2_find_past_conversation_test.rs`:**
- `rws_1_list_shows_all_projects_in_storage` — `.list` stdout includes all project paths
- `rws_2_search_by_keyword_finds_matching_sessions` — `.search query::authentication` hits project with that content
- `rws_3_project_filter_restricts_search_to_one_project` — `.search query:: project::A` excludes project B
- `rws_4_session_metadata_filters_narrow_listing` — `.list sessions::1 min_entries::2 agent::0` filters correctly
- `rws_5_show_session_displays_full_session_details` — `.show session_id::` stdout non-empty, exit 0

**US-3 (5 tests) — `tests/us3_export_session_for_review_test.rs`:**
- `rws_1_export_as_markdown_writes_output_file` — `.export session_id:: output::` creates `.md` file with content
- `rws_2_export_as_json_produces_json_output` — `.export format::json output::` creates valid JSON file
- `rws_3_export_as_text_produces_plain_text_transcript` — `.export format::text output::` creates text file, no `#` headings
- `rws_4_missing_session_id_exits_with_error` — `.export output::` without `session_id::` exits 1, stderr non-empty
- `rws_5_missing_output_exits_with_error` — `.export session_id::` without `output::` exits 1, stderr non-empty

**US-4 (5 tests) — `tests/us4_query_storage_programmatically_test.rs`:**
- `rws_1_status_verbosity_0_outputs_key_value_pairs` — stdout matches `projects: 2, sessions: 5` pattern
- `rws_2_count_outputs_bare_integer` — `.count` stdout is a plain integer, parseable as `u64`
- `rws_3_count_target_specifies_what_to_count` — `.count target::sessions` and `.count target::projects` return different values
- `rws_4_path_scopes_query_to_alternate_storage_root` — `.status path::alt` shows different counts than default
- `rws_5_nonexistent_storage_root_exits_nonzero` — `.status path::/nonexistent/path` exits non-zero

**US-5 (5 tests) — `tests/us5_resume_claude_session_test.rs`:**
- `rws_1_project_exists_exits_0_when_project_has_history` — `.project.exists path::{dir}` exits 0
- `rws_2_project_exists_exits_1_when_project_has_no_history` — `.project.exists path::{empty}` exits 1
- `rws_3_project_path_outputs_encoded_storage_path` — `.project.path project::{dir}` stdout non-empty, matches encoded form
- `rws_4_session_dir_outputs_session_working_directory_path` — `.session.dir session_id::X topic::Y` stdout non-empty
- `rws_5_session_ensure_creates_directory_and_reports_strategy` — `.session.ensure` creates dir on disk; stdout line 1 = path, line 2 = strategy

## Out of Scope

- Parameter edge case tests (EC-N) — covered by `parameter_validation_test.rs` and per-command tests
- Command integration tests (INT-N) — covered by existing `*_command_test.rs` files
- `feature_cli_tool_test.rs` (FT-N) or `operation_migration_guide_test.rs` (OP-N) — do not modify
- New CLI behavior — implement existing documented behavior only; no source changes to `src/`

## Work Procedure

1. **Verify fixture helpers.** Read `tests/common/mod.rs` to understand `clg_cmd()`,
   `write_path_project_session()`, `write_test_session()`, and any session-content helpers
   that write JSONL entries with searchable text. Identify which helpers to reuse.

2. **Implement US-1 tests** (`tests/us1_audit_session_history_test.rs`):
   - Copy module header pattern from `feature_cli_tool_test.rs` (Source, Coverage comments)
   - Implement each `rws_N_*` function using `TempDir`, `clg_cmd()`, `assert_exit()`, stdout/stderr assertions
   - Run `cargo nextest run --test us1_audit_session_history_test` → must pass 5/5

3. **Implement US-2 tests** (`tests/us2_find_past_conversation_test.rs`):
   - `rws_2` and `rws_3` require sessions with text content; use a content-writing helper or
     create entries inline; verify `.search query::authentication` hits only the seeded session
   - Run `cargo nextest run --test us2_find_past_conversation_test` → must pass 5/5

4. **Implement US-3 tests** (`tests/us3_export_session_for_review_test.rs`):
   - Use `TempDir` for output file path to avoid `/tmp` residue; assert file exists + non-empty
   - `rws_2` JSON test: parse stdout or file content with `serde_json::from_str` to assert validity
   - Run `cargo nextest run --test us3_export_session_for_review_test` → must pass 5/5

5. **Implement US-4 tests** (`tests/us4_query_storage_programmatically_test.rs`):
   - `rws_1` machine-readable: assert stdout matches `^projects: \d+, sessions: \d+$` with `regex` or manual parse
   - `rws_2` bare integer: `stdout.trim().parse::<u64>()` must succeed
   - Run `cargo nextest run --test us4_query_storage_programmatically_test` → must pass 5/5

6. **Implement US-5 tests** (`tests/us5_resume_claude_session_test.rs`):
   - `.project.exists` requires `path::` pointing to the actual project dir on disk
   - `.session.ensure` must verify directory creation on disk: `assert!(session_dir.exists())`
   - Run `cargo nextest run --test us5_resume_claude_session_test` → must pass 5/5

7. **Full verification:**
   ```
   RUSTFLAGS="-D warnings" cargo nextest run --all-features
   ```
   All tests pass with zero warnings.

8. **Update `tests/readme.md`** — add 5 new rows to the test file index table.

## Test Matrix

| Story | RWS-N | Command(s) | Fixture | Expected Behavior |
|-------|-------|-----------|---------|-------------------|
| US-1 | RWS-1 | `.status` | 2 projects, 3 sessions | stdout contains project count + session count |
| US-1 | RWS-2 | `.status verbosity::2` | 2 projects, 2 sessions each | stdout contains per-project section with counts |
| US-1 | RWS-3 | `.status verbosity::0` | 2 projects, 5 sessions | stdout = `projects: 2, sessions: 5` (parseable) |
| US-1 | RWS-4 | `.count target::sessions` | 3 sessions across 2 projects | stdout = `3` (bare integer, trimmable) |
| US-1 | RWS-5 | `.status path::{alt}` | separate alt fixture with 1 project | stdout reflects alt fixture, not default |
| US-2 | RWS-1 | `.list` | 3 projects with distinct path names | stdout lists all 3 project identifiers |
| US-2 | RWS-2 | `.search query::authentication` | project A has "authentication" in entries | stdout includes project A session; project B absent |
| US-2 | RWS-3 | `.search query::auth project::A` | 2 projects | stdout limited to project A results only |
| US-2 | RWS-4 | `.list sessions::1 min_entries::2 agent::0` | 2 sessions (1 with 2+ entries, 1 with 0) | only qualifying session shown |
| US-2 | RWS-5 | `.show session_id::{id}` | 1 session with 2 entries | stdout non-empty; exit 0 |
| US-3 | RWS-1 | `.export session_id:: output::{file}.md` | 1 session with 2 entries | output file exists, contains markdown |
| US-3 | RWS-2 | `.export format::json output::{file}.json` | 1 session with 1 entry | output file is valid JSON |
| US-3 | RWS-3 | `.export format::text output::{file}.txt` | 1 session with 2 entries | output file has no `#` or `**` markdown syntax |
| US-3 | RWS-4 | `.export output::{file}` (no `session_id::`) | any | exit 1; stderr non-empty |
| US-3 | RWS-5 | `.export session_id::X` (no `output::`) | any | exit 1; stderr non-empty |
| US-4 | RWS-1 | `.status verbosity::0` | 2 projects, 5 sessions | stdout matches `projects: 2, sessions: 5` exactly |
| US-4 | RWS-2 | `.count` | 3 sessions | stdout.trim().parse::<u64>() succeeds |
| US-4 | RWS-3 | `.count target::sessions` vs `.count target::projects` | 3 sessions, 2 projects | different integer outputs per target |
| US-4 | RWS-4 | `.status path::{alt}` | alt fixture with different counts | stdout counts differ from primary fixture |
| US-4 | RWS-5 | `.status path::/nonexistent` | no fixture | exit code ≠ 0 |
| US-5 | RWS-1 | `.project.exists path::{dir}` | project dir has 1 session | exit 0 |
| US-5 | RWS-2 | `.project.exists path::{empty_dir}` | no sessions for that project | exit 1 |
| US-5 | RWS-3 | `.project.path project::{dir}` | any | stdout non-empty; contains encoded path characters |
| US-5 | RWS-4 | `.session.dir session_id::X topic::Y` | any | stdout non-empty; path contains session id and topic |
| US-5 | RWS-5 | `.session.ensure session_id::X topic::Y` | any | directory created on disk; stdout line 2 = `resume` or `fresh` |

## Affected Entities

| Entity | Change | Path |
|--------|--------|------|
| `tests/us1_audit_session_history_test.rs` | ✨ Create | New Rust test file (5 RWS-N functions) |
| `tests/us2_find_past_conversation_test.rs` | ✨ Create | New Rust test file (5 RWS-N functions) |
| `tests/us3_export_session_for_review_test.rs` | ✨ Create | New Rust test file (5 RWS-N functions) |
| `tests/us4_query_storage_programmatically_test.rs` | ✨ Create | New Rust test file (5 RWS-N functions) |
| `tests/us5_resume_claude_session_test.rs` | ✨ Create | New Rust test file (5 RWS-N functions) |
| `tests/readme.md` | ✏️ Update | Add 5 rows to test file index |

## Related Documentation

**Feature specs:**
- `docs/feature/001_cli_tool.md` — CLI tool architecture and command catalog
- `docs/cli/user_story/001_audit_session_history.md` — Acceptance criteria for US-1
- `docs/cli/user_story/002_find_past_conversation.md` — Acceptance criteria for US-2
- `docs/cli/user_story/003_export_session_for_review.md` — Acceptance criteria for US-3
- `docs/cli/user_story/004_query_storage_programmatically.md` — Acceptance criteria for US-4
- `docs/cli/user_story/005_resume_claude_session.md` — Acceptance criteria for US-5

**Test surface specs (define RWS-N case details):**
- `tests/docs/cli/user_story/01_audit_session_history.md`
- `tests/docs/cli/user_story/02_find_past_conversation.md`
- `tests/docs/cli/user_story/03_export_session_for_review.md`
- `tests/docs/cli/user_story/04_query_storage_programmatically.md`
- `tests/docs/cli/user_story/05_resume_claude_session.md`

**Command specs (detail behavior being tested):**
- `docs/cli/command/01_status.md`, `02_list.md`, `03_show.md`, `04_count.md`, `05_search.md`
- `docs/cli/command/06_export.md`, `07_projects.md`, `08_project_path.md`
- `docs/cli/command/09_project_exists.md`, `10_session_dir.md`, `11_session_ensure.md`

**Existing test peers (patterns to follow):**
- `tests/feature_cli_tool_test.rs` — FT-N tests (structure template)
- `tests/operation_migration_guide_test.rs` — OP-N tests (structure template)

## History

- **[2026-05-25]** `CREATED` — Implement 25 RWS-N user story acceptance tests in Rust, one file per story, covering all 5 user story test spec files added during the cli/user_story/ entity normalization session.

## Verification Record

- **Date:** 2026-05-25
- **Method:** 4 independent Agent subagents, single parallel dispatch (no session context shared)
- **Scope Coherence:** PASS — In Scope: 25 named test functions across 5 files; Out of Scope: 4 named exclusions; no overlap; observable end-state defined
- **MOST Goal Quality:** PASS — Motivated (25-test gap stated); Observable (5 completion conditions with commands); Scoped (exact file paths + function names); Testable (specific `cargo nextest run` commands)
- **Value / YAGNI:** PASS — Concrete gap exists now (spec files present, Rust files absent); all 25 tests map 1:1 to named RWS-N IDs in committed spec files; no speculative items
- **Implementation Readiness:** PASS — All 5 spec files confirmed to exist; pattern file (`feature_cli_tool_test.rs`) confirmed with `mod common`, `TempDir`, `clg_cmd()` infrastructure; 25-row Test Matrix; executable Work Procedure steps
