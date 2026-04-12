# Redesign `.projects` output as project-centric summary and list

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)

## Goal

Redesign `projects_routine` output so that project directories are the primary display unit in both summary and list modes, replacing the current session-UUID enumeration with aggregated project summaries. (Motivated: the current output lists 17 raw session UUIDs when a user has worked in one directory for a week — the user cares about the PROJECT, not which of the 17 JSONL files is relevant; Observable: bare `clg .projects` outputs `Active project  ~/path  (N sessions, last active Xago)` instead of `Active session  {8-char-id}  ...`; `clg .projects scope::under` shows one line per project directory, not one line per session file; Scoped: only `projects_routine` output rendering rewritten — scope logic, filter parameters, `is_default` guard, and `verbosity::` mode-selection guard all unchanged; Testable: `clg .projects | grep "Active project"` → non-empty AND `cargo nextest run --test projects_output_format_test` → 0 failures.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` § `projects_routine` — rewrite output rendering; add `ProjectSummary` aggregation struct and `aggregate_projects` helper function
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_output_format_test.rs` — add new tests for project-centric output format
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_command_test.rs` — update `it1` (summary mode) test assertions to match new `Active project` header
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` — rewrite Command `.projects` section: new output format description, updated verbosity matrix, updated examples
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/projects.md` — add test cases for new output format
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/dictionary.md` § `Active Session` — update to `Active Project` concept (or add a new `Active Project` entry)

## Out of Scope

- Scope logic, filter logic, `is_default` guard — unchanged (task 012 fix stays intact)
- `verbosity::` mode-selection guard — unchanged (verbosity is still a display modifier only)
- Task 015 rename — must be completed before this task
- `.list` command — it remains project-first for global navigation; `.projects` becomes the session-aware project-summary command for current-work context

## Description

After task 015, `.projects` behaves identically to the old `.sessions`. The next step is to make the output actually project-centric. The core insight: a user who has worked in `/home/user/pro/myapp` for a week has 17 session JSONL files there — one per Claude Code run. Listing all 17 with their UUIDs is noise. The user wants to know: "what project did I work on?" and "when?"

**Summary mode** (bare `clg .projects`, `is_default` = true):

```
Active project  ~/path/to/project  (17 sessions, last active 2h ago)
Last session:  a1b2c3d4  2h  43 entries
Last message:  {truncated text}
```

No sessions in scope → `No active project found.`

**List mode** (`scope::`, `session::`, `agent::`, `min_entries::`, or `limit::` given):

```
Found 3 projects:

  * ~/path/to/project-a  (17 sessions, last active 2h ago)
  - ~/path/to/project-b  (5 sessions, last active 1d ago)
  - ~/path/to/project-c  (1 session, last active 3d ago)
```

At `verbosity::0`: project paths only (one per line, machine-readable).
At `verbosity::2+`: each project expands to show most-recent session detail.

The `aggregate_projects` helper groups sessions by project path, computes session count, last mtime, and last message — producing a `Vec<ProjectSummary>` sorted by last mtime descending. The existing session loading and filtering infrastructure is reused; only the rendering changes.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- TDD: write failing tests BEFORE implementing; confirm they fail; then implement
- `is_default` guard must not change — summary mode still triggers on bare invocation (task 012 invariant)
- `verbosity::` must still not affect mode selection — verbosity is display modifier only
- `verbosity::0` must output machine-readable project paths (one per line)
- `aggregate_projects` must be under 50 lines; `ProjectSummary` struct must be in `mod private`
- No duplication: reuse existing session loading infrastructure from core library

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` on struct sizing, `test_organization.rulebook.md` on test doc comments.
2. **Read current output implementation** — Read `src/cli/mod.rs` `projects_routine` output blocks: the `if is_default` branch (summary mode) and the `else` branch (list mode). Understand the session data types available.
3. **Design `ProjectSummary`** — Draft the struct in `mod private`:
   ```rust
   struct ProjectSummary
   {
     path           : PathBuf,
     session_count  : usize,
     last_mtime     : SystemTime,
     last_session_id: String,
     last_entry_count: usize,
     last_message   : Option< String >,
   }
   ```
4. **Write Test Matrix** — populate every row before writing any test code. See Test Matrix below.
5. **Write failing tests** — Add the following to `tests/projects_output_format_test.rs`:
   - `it_summary_mode_shows_active_project_header` — bare invocation, assert `"Active project"` in stdout, assert `"Active session"` ABSENT
   - `it_summary_mode_shows_session_count` — assert stdout contains `"sessions,"` (plural aggregate)
   - `it_list_mode_one_line_per_project` — `scope::under` with 2 projects, assert output lines with `* ~/path` (project paths), assert no bare UUID lines as primary entries
   - `it_verbosity_0_shows_paths_only` — `scope::under verbosity::0`, assert output contains only paths (no "sessions," text)
   Confirm these tests fail before implementation.
6. **Implement `aggregate_projects`** — Add helper that groups `Vec<SessionInfo>` by project path, computes stats, returns `Vec<ProjectSummary>` sorted by `last_mtime` descending.
7. **Rewrite summary mode output** — Replace the current `Active session` block with `Active project` block using `ProjectSummary`.
8. **Rewrite list mode output** — Replace session-level enumeration with project-level summary lines. `verbosity::0` → paths only. `verbosity::1` → `  * ~/path  (N sessions, last active Xago)`. `verbosity::2+` → project line + indented most-recent session details.
9. **Update `it1` test** — Update `it1_default_shows_active_session_summary` in `tests/projects_command_test.rs` to assert `"Active project"` instead of `"Active session"`. (If the test name is now misleading, rename it to `it1_default_shows_active_project_summary`.)
10. **Run tests** — `cargo nextest run --test projects_output_format_test` → all new tests pass. `cargo nextest run --test projects_command_test` → all tests pass including `it1` and `it47`.
11. **Run full suite** — `w3 .test level::3` → 0 failures.
12. **Update `docs/cli/commands.md`** — Rewrite the `.projects` command Default invocation block (new `Active project` format), rewrite verbosity matrix (project-level columns), update all examples.
13. **Update `docs/cli/testing/command/projects.md`** — Add IT entries for the new format tests.
14. **Update `docs/cli/dictionary.md`** — Add `Active Project` entry: "The most-recently-modified project within the current scope. Displayed by bare `clg .projects` as a single summary block showing session count and last message." Optionally update or deprecate `Active Session` (which was specific to the old `.sessions` summary format).
15. **Walk Validation Checklist** — every answer must be YES.
16. **Update task status** — Set ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|----------------|-------------------|-------------------|
| T01 | `clg .projects` (bare) | project with ≥1 session under cwd | Summary: stdout contains `Active project`, does NOT contain `Active session` |
| T02 | `clg .projects` (bare) | project with 3 sessions | Summary: stdout contains `(3 sessions,` or `(1 session,` aggregate |
| T03 | `clg .projects scope::under` | 2 distinct project dirs | List: one `*` or `-` line per project dir; UUID lines absent as primary entries |
| T04 | `clg .projects scope::under verbosity::0` | 2 distinct project dirs | Bare project paths only (one per line); no `sessions,` text |
| T05 | `clg .projects verbosity::1` (only param) | project with sessions | Summary mode (verbosity does not activate list mode — task 012 regression guard) |
| T06 | `clg .projects scope::under verbosity::2` | project with sessions | List + session detail: most-recent session ID shown under project line |
| T07 | `clg .projects scope::under` | 0 projects in scope | `No active project found.` or empty output (not an error) |

## Acceptance Criteria

- `clg .projects` (bare) stdout contains `Active project` (not `Active session`)
- `clg .projects scope::under` shows one `* ~/path` line per unique project directory (not one per session file)
- `clg .projects scope::under verbosity::0` outputs bare paths only (one per line)
- `it1` test in `tests/projects_command_test.rs` asserts `"Active project"` and passes
- `it47` test in `tests/projects_command_test.rs` still passes (verbosity guard intact)
- All 7 Test Matrix rows have corresponding passing tests in `tests/projects_output_format_test.rs`
- `w3 .test level::3` → 0 failures
- `aggregate_projects` function present in `src/cli/mod.rs`; `ProjectSummary` struct present in `mod private`

## Validation

### Checklist

Desired answer for every question is YES.

**Summary mode output**
- [ ] Does `clg .projects` stdout contain `Active project`?
- [ ] Does `clg .projects` stdout NOT contain `Active session`?
- [ ] Does `clg .projects` stdout contain a session count aggregate (e.g., `(N sessions,`)?

**List mode output**
- [ ] Does `clg .projects scope::under` show one line per project directory (not per session file)?
- [ ] Does `clg .projects scope::under verbosity::0` output bare paths only?

**Mode selection guard (task 012 invariant)**
- [ ] Does `clg .projects verbosity::1` produce summary mode (not list mode)?
- [ ] Does `it47` pass in `tests/projects_command_test.rs`?

**Tests present**
- [ ] Do all 7 Test Matrix rows have corresponding tests in `tests/projects_output_format_test.rs`?
- [ ] Does `it1` in `tests/projects_command_test.rs` assert `Active project`?

**Implementation artifacts**
- [ ] Is `ProjectSummary` struct present in `src/cli/mod.rs` `mod private`?
- [ ] Is `aggregate_projects` function present in `src/cli/mod.rs`?

**Documentation updated**
- [ ] Does `docs/cli/commands.md` `.projects` section describe `Active project` format?
- [ ] Does `docs/cli/dictionary.md` have an `Active Project` entry?

**Out of Scope confirmation**
- [ ] Is the `is_default` guard logic unchanged (same conditions as after task 012)?
- [ ] Are all filter parameters (`scope::`, `session::`, `agent::`, `min_entries::`) still functional?

### Measurements

**M1 — summary mode shows project header**
Command: `clg .projects 2>&1 | grep "Active project" | wc -l`
Before: `0` (shows `Active session`). Expected: `1`. Deviation: `0` means implementation not applied.

**M2 — list mode groups by project**
Command: `clg .projects scope::under 2>&1 | grep -c "^\s*[*-] ~/"` (count project-level lines)
Before: counts session UUIDs. Expected: equals number of distinct project directories in scope. Deviation: count doesn't match distinct projects.

**M3 — verbosity guard intact (task 012 regression)**
Command: `cargo nextest run --test projects_command_test it47 2>&1 | tail -3`
Before: `1 passed`. Expected: `1 passed`. Deviation: failure means guard was broken.

**M4 — new format tests pass**
Command: `cargo nextest run --test projects_output_format_test 2>&1 | tail -3`
Before: tests don't exist. Expected: all pass. Deviation: any failure.

**M5 — full suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: `Summary: 13/13 crates passed, 0 failed`. Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

**AF1 — test uses real clg binary (not mocked)**
Check: `grep -c "clg_cmd\|common::clg\|run_clg" tests/projects_output_format_test.rs`
Expected: ≥ 4. Why: ensures new tests exercise the real binary output path, not mocked strings.

**AF2 — `Active session` truly replaced in summary output**
Check: `grep -c "Active session" src/cli/mod.rs`
Expected: `0`. Why: ensures the old summary header string was replaced, not duplicated or conditionally selected.

**AF3 — list mode output asserts project count not session count**
Check: `grep -c "one_line_per_project\|project_dir\|distinct_project" tests/projects_output_format_test.rs`
Expected: ≥ 1. Why: ensures the list mode test explicitly validates project-level aggregation.

## Outcomes

- `ProjectSummary` struct and `aggregate_projects` function added to `src/cli/mod.rs`
- Summary mode replaced `render_active_summary` with `render_active_project_summary` — outputs `Active project  {path}  ({N} sessions, last active {age})`
- List mode redesigned: v0 → project paths only; v1+ → `aggregate_projects` result iterated (time-sorted), family grouping preserved (P6)
- 4 new TDD tests added (IT-50..IT-53) in `tests/projects_output_format_test.rs`
- `tests/projects_command_test.rs` updated: IT-1, IT-30, IT-47 assertions aligned to new format; 11 tests updated for project-centric output and new header noun ("Found N projects:")
- Level 3 validation: 281 tests, 3 doctests, clippy 0 errors — all pass

