# manual testing plan - claude_storage v1.9.1

## responsibility

Comprehensive manual testing coverage for all CLI commands, parameter combinations, and edge cases in the claude_storage crate.

## testing scope

### commands to test

Current command set (16 commands as of v1.9.1), numbered to match `docs/cli/command/`:

1. `.status` - Storage statistics (path parameter tested in Phase 1D)
2. `.list` - Project/session listing with filtering (DEPRECATED — superseded by `.projects`)
3. `.show` - Session/project display (location-aware)
4. `.count` - Fast counting operations (target parameter tested in Phase 1A)
5. `.search` - Full-text search (parameter validation tested in Phase 1B)
6. `.export` - Export sessions to file (parameter validation tested in Phase 1C)
7. `.projects` - Project-centric listing with scope filtering (renamed from `.sessions` in task-015; redesigned in task-016; session liveness manually verified 2026-08-23 — see below)
8. `.project.path` - Print canonical storage path for a topic
9. `.project.exists` - Exit-code check whether a topic has session history
10. `.session.dir` - Print or create session directory for a topic
11. `.session.ensure` - Ensure session directory exists for a topic
12. `.tail` - Print last N conversation turns for current directory (turn-grouped renderer manually verified 2026-08-21 — see below)
13. `.usage` - Per-session usage table — turns, tokens, cache, duration, dir
14. `.rollup` - Grouped/filtered/sorted/projected token-usage table
15. `.cost` - Per-conversation cost table with agent sessions folded in
16. `.session.path` - Resolve a session's absolute transcript file path

**Removed commands (do not test):**
- `.show.project` — removed in task-013 (deprecated stub)
- `.session` — removed in task-014 (duplicate of `.project.exists`)

### parameter validation

Each command must be tested with:
- Valid parameter values (all documented options)
- Invalid parameter values (typos, wrong types, out-of-range)
- Missing required parameters
- Extra unexpected parameters
- Parameter combinations (valid and invalid)
- Edge values (0, negative, very large numbers)
- Special characters and encoding

### data conditions

- Empty storage (no projects)
- Single project (UUID type)
- Single project (path type)
- Multiple projects (mixed types)
- Projects with no sessions
- Projects with main sessions only
- Projects with agent sessions only
- Projects with mixed session types
- Sessions with 0 entries
- Sessions with 1 entry
- Sessions with thousands of entries
- Corrupted JSONL files
- Missing history.jsonl
- Nonexistent paths in storage

### path handling

- Absolute paths (`/home/user/project`)
- Relative paths (`../project`, `./project`)
- Special paths (`.`, `..`, `~`, `~/subdir`)
- Path-encoded format (`-home-user-project`)
- Path patterns (substring matching)
- Paths with spaces
- Paths with special characters
- Paths with UTF-8 characters
- Very long paths (>255 characters)
- Nonexistent paths
- Paths to deleted directories

### session id handling

- Full UUID format (`abc-123-def-456-789`)
- Partial UUID (first 8 chars: `abc-123`)
- Agent format (`agent-022ada42`)
- Invalid formats
- Nonexistent session IDs
- Empty session ID

### REQ-011 content display

- Default shows content
- show_metadata::1 shows metadata only
- metadata::1 parameter
- entries::1 parameter
- Content truncation behavior
- Empty conversation content
- Very large conversation content
- UTF-8 content handling
- Special characters in content

## corner case matrix

### `.status` command

| Test Case | Parameters | Expected Behavior | Priority |
|-----------|------------|-------------------|----------|
| Default output | (none) | Show basic statistics (fast path) | High |
| With show_tokens | `show_tokens::1` | Include token usage section | High |
| With show_stat | `show_stat::1` | Include extended statistics | High |
| Empty storage | (none) | Show 0 counts | High |
| Large storage | (none) | Performance check | Low |

### `.list` command

| Test Case | Parameters | Expected Behavior | Priority |
|-----------|------------|-------------------|----------|
| Default | (none) | List projects only | High |
| UUID projects | `type::uuid` | Filter UUID projects | High |
| Path projects | `type::path` | Filter path projects | High |
| All projects | `type::all` | Show all projects | High |
| Invalid type | `type::invalid` | Error message | Medium |
| Explicit sessions enable | `sessions::1` | Show sessions | High |
| Explicit sessions disable | `sessions::0` | Hide sessions | High |
| Session filter auto-enable | `session::commit` | Auto-show sessions | High |
| Agent filter auto-enable | `agent::1` | Auto-show sessions | High |
| Min entries filter auto-enable | `min_entries::10` | Auto-show sessions | High |
| Override auto-enable | `sessions::0 session::test` | Projects only | High |
| Path current dir | `path::.` | Match current dir | High |
| Path parent dir | `path::..` | Match parent dir | High |
| Path home | `path::~` | Match home dir | High |
| Path home + subdir | `path::~/pro` | Match home subdir | High |
| Path absolute | `path::/home/user/pro` | Match absolute | High |
| Path relative | `path::../lib` | Resolve + match | High |
| Path pattern | `path::assistant` | Substring match | High |
| Path nonexistent | `path::/nonexistent` | No matches | Medium |
| Agent main only | `agent::0` | Filter main sessions | Medium |
| Agent sub only | `agent::1` | Filter agent sessions | Medium |
| Min entries zero | `min_entries::0` | All sessions | Medium |
| Min entries high | `min_entries::1000` | Few/no matches | Medium |
| Min entries negative | `min_entries::-5` | Error or 0 | Medium |
| Session substring | `session::commit` | Match session IDs | High |
| Session empty | `session::` | All or error | Low |
| Session nonexistent | `session::xyz999` | No matches | Medium |
| Combined filters | `path::assistant session::default agent::0 min_entries::5` | All filters apply | High |
| Empty storage | (none) | No projects | High |

### `.show` command

| Test Case | Parameters | Expected Behavior | Priority |
|-----------|------------|-------------------|----------|
| No params (in project dir) | (none) | Show current project | High |
| No params (not in project) | (none) | Error or empty | High |
| Session only (in project) | `session_id::abc123` | Show session in current project | High |
| Session only (not in project) | `session_id::abc123` | Error or search | Medium |
| Project only (absolute path) | `project::/home/user/pro` | Show all sessions | High |
| Project only (path-encoded) | `project::-home-user-pro` | Show all sessions | High |
| Project only (UUID) | `project::abc-123-def` | Show all sessions | High |
| Project only (Path format) | `project::Path("/home/user/pro")` | Show all sessions | High |
| Project only (current dir) | `project::.` | Show current project | Medium |
| Project only (parent dir) | `project::..` | Show parent project | Medium |
| Project only (home) | `project::~` | Show home project | Medium |
| Both params (full UUID) | `session_id::abc-123-def project::/path` | Show specific session | High |
| Both params (partial UUID) | `session_id::abc123 project::/path` | Show specific session | High |
| Both params (agent ID) | `session_id::agent-022ada42 project::/path` | Show agent session | High |
| Nonexistent session | `session_id::nonexistent project::/path` | Error message | High |
| Nonexistent project | `project::/nonexistent` | Error message | High |
| Invalid session format | `session_id::123` | Error message | Medium |
| Invalid project format | `project::???` | Error message | Medium |
| Default (content) | `session_id::abc` | Full content | High |
| Metadata parameter | `metadata::1 session_id::abc` | Metadata only | High |
| Entries parameter | `entries::1 session_id::abc` | Show all entries | Medium |
| Empty session (0 entries) | `session_id::{empty} project::/path` | Handle gracefully | High |
| Large session (1000+ entries) | `session_id::{large} project::/path` | Performance + truncation | Medium |
| UTF-8 in content | `session_id::{utf8} project::/path` | Display correctly | High |

### `.projects` command

| Test Case | Parameters | Expected Behavior | Priority |
|-----------|------------|-------------------|----------|
| Default (summary mode) | (none) | Show active project summary | High |
| Scope local | `scope::local` | List only current project | High |
| Scope under | `scope::under` | List projects under CWD | High |
| Scope relevant | `scope::relevant` | List ancestor projects | High |
| Scope global | `scope::global` | List all projects | High |
| With path filter | `path::/home/user/pro` | Scoped to path | High |
| With session filter | `session::commit` | Filter by session ID | High |
| With agent filter | `agent::1` | Agent sessions only | High |
| Agent filter off | `agent::0` | Non-agent sessions only | High |
| Min entries filter | `min_entries::5` | Sessions with ≥5 entries | High |
| Limit default | `scope::global` | Capped at default limit | High |
| Limit explicit | `scope::global limit::5` | Max 5 sessions per project | High |
| Limit zero (unlimited) | `scope::global limit::0` | All sessions listed | High |
| Default (family summary) | `scope::global` | Family summary with agent collapse | High |
| Tree mode | `scope::global show_tree::1` | Tree-indented agent display | High |
| Zero-byte sessions | project with only placeholders | Project excluded from list | High |
| Zero-byte mixed | project with real + placeholder | Count excludes zero-byte | High |
| Invalid scope | `scope::invalid` | Error message | Medium |
| Negative min_entries | `min_entries::-1` | Error message | Medium |

### `.project.path` / `.project.exists` / `.session.dir` / `.session.ensure` commands

| Test Case | Parameters | Expected Behavior | Priority |
|-----------|------------|-------------------|----------|
| `.project.path` default topic | (none) | Print default topic path | High |
| `.project.path` custom topic | `topic::mytopic` | Print topic path | High |
| `.project.path` invalid topic (with /) | `topic::a/b` | Error message | High |
| `.project.exists` present | `topic::` with history | Exit 0 | High |
| `.project.exists` absent | `topic::` no history | Exit 1 + "no sessions" | High |
| `.session.dir` create | `topic::new` | Create + print dir | High |
| `.session.ensure` idempotent | existing topic | Exit 0, no duplicate | High |

### `.count` command

| Test Case | Parameters | Expected Behavior | Priority | Automated |
|-----------|------------|-------------------|----------|-----------|
| Count projects | `target::projects` | Project count | High | ✅ Phase 1A |
| Count sessions (no project) | `target::sessions` | Error (needs project) | High | ✅ Phase 1A |
| Count sessions (with project) | `target::sessions project::/path` | Session count | High | |
| Count entries (no session) | `target::entries` | Error (needs session) | High | ✅ Phase 1A |
| Count entries (with session) | `target::entries session::abc project::/path` | Entry count | High | |
| Invalid target | `target::invalid` | Error message | High | ✅ Phase 1A |
| Missing target | (none) | Error message | High | |
| Nonexistent project | `target::sessions project::/nonexistent` | Error message | High | |
| Nonexistent session | `target::entries session::nonexistent project::/path` | Error message | High | |
| Empty project (0 sessions) | `target::sessions project::/empty` | Count = 0 | Medium | |
| Empty session (0 entries) | `target::entries session::empty project::/path` | Count = 0 | Medium | |

### `.search` command

| Test Case | Parameters | Expected Behavior | Priority | Automated |
|-----------|------------|-------------------|----------|-----------|
| Missing query | (none) | Error message | High | ✅ Phase 1B |
| Empty query | `query::` | Error message | High | ✅ Phase 1B |
| Simple query | `query::error` | Find matches | High | |
| Case sensitive search | `query::Error case_sensitive::1` | Case-sensitive matches | High | ✅ Phase 1B |
| Case insensitive search | `query::error case_sensitive::0` | Case-insensitive matches | High | |
| Filter by entry_type user | `query::test entry_type::user` | User messages only | High | ✅ Phase 1B |
| Filter by entry_type assistant | `query::test entry_type::assistant` | Assistant messages only | High | |
| Filter by entry_type all | `query::test entry_type::all` | All entries | High | |
| Invalid entry_type | `query::test entry_type::invalid` | Error message | High | ✅ Phase 1B |
| Filter by project | `query::test project::/path` | Project-specific results | High | |
| Filter by session | `query::test session::abc` | Session-specific results | High | |
| Unknown parameter | `query::test unknown::value` | Error message | High | ✅ Phase 1B |
| Nonexistent project | `query::test project::/nonexistent` | No results or error | Medium | |
| Nonexistent session | `query::test session::nonexistent` | No results or error | Medium | |
| No matches | `query::xyz999` | Empty result set | Medium | |
| UTF-8 query | `query::日本語` | Unicode search | Medium | |
| Special chars query | `query::foo*bar` | Literal or pattern | Medium | |

### `.export` command

| Test Case | Parameters | Expected Behavior | Priority | Automated |
|-----------|------------|-------------------|----------|-----------|
| Missing session_id | `output::/tmp/test.md` | Error message | High | ✅ Phase 1C |
| Missing output | `session_id::abc` | Error message | High | ✅ Phase 1C |
| Invalid format | `session_id::abc output::/tmp/test format::csv` | Error message | High | ✅ Phase 1C |
| Format markdown | `session_id::abc output::/tmp/test.md format::markdown` | Export to markdown | High | |
| Format json | `session_id::abc output::/tmp/test.json format::json` | Export to JSON | High | |
| Format text | `session_id::abc output::/tmp/test.txt format::text` | Export to plain text | High | |
| Default format | `session_id::abc output::/tmp/test.md` | Defaults to markdown | High | |
| Nonexistent session | `session_id::nonexistent output::/tmp/test.md` | Error message | High | |
| Nonexistent directory | `session_id::abc output::/nonexistent/dir/test.md` | Error message | High | |
| File exists | `session_id::abc output::/existing/file.md` | Overwrite or error | Medium | |
| Filter by project | `session_id::abc project::/path output::/tmp/test.md` | Export from specific project | High | |
| Permission denied | `session_id::abc output::/root/test.md` | Error message | Low | |

### general edge cases

| Test Case | Scenario | Expected Behavior | Priority |
|-----------|----------|-------------------|----------|
| No ~/.claude/ directory | Fresh system | Error message | High |
| Empty ~/.claude/projects/ | No projects | Empty results | High |
| Corrupted JSONL | Malformed JSON | Skip + warning | High |
| Missing history.jsonl | Deleted file | Warn + continue | Medium |
| UTF-8 paths | Non-ASCII dirs | Handle correctly | High |
| Spaces in paths | `/home/user/my project/` | Handle correctly | High |
| Very long path | 300+ chars | Handle correctly | Low |
| Special chars | `path::/home/$USER/test` | Handle correctly | Medium |
| Case sensitivity | `session::ABC` vs `session::abc` | Case-insensitive | Medium |
| Whitespace in params | `session:: abc ` | Trim or error | Medium |
| Empty param values | `session::` | Handle gracefully | Medium |
| Multiple same params | `show_tokens::0 show_tokens::1` | Last wins or error | Low |
| Unknown params | `unknown::value` | Ignore or error | Medium |
| Storage permission denied | No read access | Error message | Low |

## test execution plan

### phase 1: automated test coverage audit (pre-manual)

Before manual testing, verify automated test coverage:

1. ✅ Review all existing tests (90 tests: 81 passing, 9 ignored)
2. ✅ Identify which corner cases are already covered (Phase 1 complete - see tests/readme.md)
3. ✅ Create gap analysis (Parameter Coverage Matrix in -current_plan.md shows 58% coverage)
4. Focus manual testing on uncovered areas (integration tests, edge cases, performance)

**Phase 1 Automated Coverage Achievements**:
- `.status` path parameter: 4/5 tests passing (1 ignored for default path)
- `.count` target parameter: 4 comprehensive validation tests
- `.search` parameters: 5/8 validation tests passing (3 integration tests ignored)
- `.export` parameters: 3/8 validation tests passing (5 integration tests ignored)
- 1 bug found and fixed (Finding #010: parameter validation — verbosity since removed)

**Remaining Manual Test Focus**:
- Integration tests for .search (project, session, entry_type parameters)
- Integration tests for .export (format, project parameters)
- .list path and session integration tests
- Performance testing (large storage, large sessions)
- Error handling (corrupted files, permissions, encoding)

### phase 2: command validation

For each command:

1. Test all valid parameter combinations
2. Test all invalid parameter combinations
3. Test all edge values (0, negative, very large)
4. Test parameter type errors

### phase 3: data condition testing

1. Setup test storage with various conditions
2. Run commands against each condition
3. Verify behavior matches spec
4. Document any unexpected behavior

### phase 4: integration testing

1. Test command sequences (`.list` → `.show`)
2. Test copy-paste workflows
3. Test scripting scenarios
4. Test REPL vs one-shot behavior differences

### phase 5: performance testing

1. Large storage (100+ projects)
2. Large sessions (1000+ entries)
3. Deep nesting
4. Many agent sessions

### phase 6: error handling

1. Corrupted files
2. Missing files
3. Permission issues
4. Invalid encoding
5. Disk full scenarios

## test result documentation

For each test executed:

1. Record command executed
2. Record actual output
3. Compare against expected behavior
4. Mark as PASS/FAIL
5. For failures:
   - Document expected vs actual
   - Create bug reproducer test
   - Fix issue properly (no workarounds)
   - Verify fix with ctest3
   - Re-run manual test

## test metrics

- Total corner cases identified: ~100+
- High priority cases: ~60
- Medium priority cases: ~30
- Low priority cases: ~10+

## test status

**Status**: IN PROGRESS — phase 1-6 comprehensive manual testing completed 2026-03-13; issue-025..028 found and fixed 2026-03-29; issue-034 found and fixed 2026-04-12 (zero-byte session count mismatch in `.projects`)

**Last Updated**: 2026-04-12

**Test Run Log**: execution writes a hyphen-prefixed results file into this directory. It is a
temporary artifact — gitignored, freely deletable, and deliberately not linked from here.

## bugs found and fixed (2026-03-13 manual testing session)

All bugs found during `/test_manual` execution. Each has a bug reproducer test.

| Issue | Description | Fix Location | Test File |
|-------|-------------|--------------|-----------|
| #015 | `.status` performance: >2min with 1903 projects (O(total JSONL) at default output) | `storage.rs::global_stats_fast()`, `cli/status.rs` | `status_global_stats_fast_bug.rs` |
| #016 | `count_entries()` counted all JSONL lines (metadata + conversation), not just user/assistant | `session.rs::count_entries()` | `count_entries_bug.rs` |
| #017 | `.count` failed with "Failed to count entries" when CWD project had any corrupted session | `cli/mod.rs::count_routine()` loop | `count_command_bug_fix.rs` |
| #018 | issue-016 fix (full JSON parse in `count_entries()`) caused `.list min_entries::N` to SIGTERM | `session.rs::count_entries()` (string-search approach) | `count_entries_bug.rs` / `list_smart_session_display.rs` |
| #019 | `.export format::xml` showed "I/O error during unknown operation" instead of format hint | `export.rs::ExportFormat::from_str()` | `export.rs::export_format_invalid_string_returns_clear_error` |
| #025 | `Found 1 sessions:`/`Found 1 projects:`/`Found 1 matches:` used wrong plural form for count==1 | `cli/mod.rs` (3 writeln! calls) | `sessions_command_test.rs` IT-14..IT-16, `list_command_test.rs`, `search_command_test.rs` |
| #026 | `.export` to nonexistent directory: "I/O error during unknown operation" — missing output path context | `claude_storage_core/src/export.rs::export_session_to_file()` | `export_command_test.rs::test_export_output_path_in_error_message` |
| #027 | `.list sessions::1` shows `(1 sessions)` — wrong plural in per-project session count label | `cli/list.rs` | `list_command_test.rs::test_list_session_count_singular_when_one_session` |
| #028 | `.show` session header shows `(1 entries)` and `.show.project` shows `(1 entries, last:)` — wrong plural for irregular noun "entry" | `cli/mod.rs::show_session_routine()` + `show_project_routine()` | `smart_show_command.rs::test_show_session_single_entry_header_says_entry_not_entries`, `show_project_command.rs::test_show_project_single_entry_session_says_entry_not_entries` |
| #034 | `.projects` list mode: header showed `(2 sessions)` but rendered 0 lines when project had only zero-byte placeholder sessions. Same root cause in flat branch and summary mode. | `cli/mod.rs` 3 sites: `aggregate_projects`, use_families `root_count`, flat `group_count` | `projects_zero_byte_count_bug.rs` IT-54..IT-56 |

## output toggle behavior reference

`.status` command (post issue-015 fix):

| Toggle | Mode | Speed | Shows |
|--------|------|-------|-------|
| (default) | Fast (filesystem only) | ~50ms | Projects + sessions |
| `show_tokens::1` | Full (JSONL parsing) | ~minutes | Above + entry counts + token usage |
| `show_stat::1` | Extended stats | varies | Additional statistics |

## corner cases verified (2026-03-13)

All PASS unless noted:

| Command | Test Case | Result |
|---------|-----------|--------|
| `.status` | default fast path | ✅ PASS ~50ms |
| `.status` | show_tokens::1 full parse | ✅ PASS |
| `.count` | target::entries empty session | ✅ PASS returns 0 |
| `.count` | target::entries metadata-only session | ✅ PASS returns 0 |
| `.count` | in project with corrupted session | ✅ PASS warns + skips |
| `.export` | format::text | ✅ PASS |
| `.export` | format::json | ✅ PASS |
| `.export` | format::markdown | ✅ PASS |
| `.export` | format::xml (invalid) | ✅ PASS clear error with valid options |
| `.export` | overwrite existing file | ✅ PASS silently overwrites |
| `.export` | missing session_id | ✅ PASS required-arg error |
| `.export` | missing output path | ✅ PASS required-arg error |
| `.list` | path::~/pro (tilde expansion) | ✅ PASS |
| `.list` | path::/abs/path (substring filter) | ✅ PASS |
| `.list` | min_entries::10 performance | ✅ PASS ~35s with 2429 sessions |
| `.search` | query with spaces | ⚠️ pre-existing: unilang splits at spaces |
| paths with spaces | any command | ⚠️ pre-existing: unilang splits at spaces |

## `.tail` turn-grouped renderer — manual session (2026-08-21)

Ran against the real local store (`~/.claude/projects/`), inside the runbox container, on session `feed0009` (357 entries → 229 turns) plus two other real projects. 24 cases; the two failures found were fixed and covered by regression tests before this record was written.

### Why manual testing was needed here

Integration tests assert on synthetic fixtures, which is exactly what makes them cheap and exactly what makes them blind to real-store distributions. Three of the findings below could not have come from a fixture: the tool-summary gap is a property of *which tools this store actually uses*, the empty `.show` body is a property of *how real sessions interleave tool results*, and width discipline only means something measured against real paths and real commands.

### Results

| # | Case | Result |
|---|------|--------|
| M1 | Zero-arg default | ✅ 4 turns, header + rule lines, 27 lines |
| M2 | Trailing bytes | ✅ ends with exactly one `\n` — no stray blank line |
| M3 | Chrome width in characters | ✅ rule lines exactly 76, tool lines exactly 76, fold hint 60, header 63, compact rows ≤76; nothing overflows |
| M4 | ANSI absence when piped | ✅ zero escape sequences in redirected output |
| M5 | `last::1` | ✅ 1 turn; header uses the singular form `turn 229 of 229` |
| M6 | `l::` alias | ✅ byte-identical to `last::` |
| M7 | `full::1` vs default | ✅ 23 lines / 1 fold → 144 lines / 0 folds over the same window |
| M8 | `compact::1 last::12` | ✅ exactly 12 rows, zero rule lines, widest row 76 chars |
| M9 | `compact::1 full::1` | ✅ byte-identical to `compact::1` — `full::` inert, as documented |
| M10 | `last::0` (whole session) | ✅ 229 turns, 862 lines, sub-second |
| M11 | `last::0 compact::1` | ✅ 229 rows, sub-second — a 12-day session on two screens |
| M12 | Header span vs actual | ✅ `turns 1-229 of 229` matches 229 rule lines exactly |
| M13 | `last::-1` / `last::abc` | ✅ exit 1 both; `last must be non-negative` / unilang coercion error |
| M14 | Nonexistent `topic::` | ✅ exit 1, `Session not found for topic: …` |
| M15 | `path::/etc` (no project) | ✅ exit 2, `No project found for path: /etc` |
| M16 | `path::` to another real project | ✅ resolves and renders; project label switches to `kit` |
| M17 | Fold-hint round trip | ✅ the emitted `clg .show session_id::feed0009 index::616` runs and lands on the folded entry |
| M18 | `.show` regression | ⚠️ **defect found** — see below; ✅ after fix |
| M19 | Tool-line integrity | ⚠️ **defect found** — see below; ✅ 0/212 bare after fix |
| M20 | Result annotations | ✅ `↳ 1 line` ×97, `↳ 56 lines`, `↳ error` ×6 — plural/singular both correct |
| M21 | Unmodelled blocks | ✅ 0 in this session (the `⧉` path is covered by INT-21 instead) |
| M22 | Colour on a pty / `NO_COLOR` | ✅ colour present under `script(1)`; `NO_COLOR=1` suppresses it |
| M23 | `path::` above any project root | ✅ exit-2 path, no crash |

### Defects found and fixed

| # | Symptom | Root cause | Fix | Regression test |
|---|---------|------------|-----|-----------------|
| T1 | `.show last::2` printed `2026-08-21 10:31 · User:` over a blank line | Chat-log mode suppresses successful `tool_result` blocks. That was written when only assistant entries reached the renderer; once user entries did too, a tool-result-only user record rendered to nothing under a header. | `format_entry_content` emits `↳ tool result` when the body renders empty, naming which kind of block was suppressed | `cli_cmd_show_test.rs::int_24_show_marks_a_tool_result_only_entry_instead_of_leaving_it_blank` |
| T2 | 22 of 212 tool lines rendered as a bare `⚙ TaskUpdate` with no summary | `TOOL_SUMMARY_KEYS` was drawn from the file/shell/web tools; none of the task tools carry any of those keys. A store-wide survey put this at 5.1% of all tool calls, 87% of them `TaskUpdate`. | Appended `status`, `recipient`, `taskId`, `task_id`; `status` deliberately outranks `taskId` so the line reads `completed`, not `42` | `cli_cmd_tail_test.rs::int_24_task_tool_summarises_by_status_not_id` |

Residual bare `⚙ Name` lines are intentional: `TaskList` takes no input at all, and `TodoWrite`/`AskUserQuestion` carry only structured arrays with no single telling string. Store-wide this is 0.7% of tool calls, down from 5.1%.

### Reproducing

The battery is a disposable script, not a committed fixture — it reads the operator's own store, so its expected values are local. To rebuild it, drive `clg .tail` through the case list above via `runbox .live` and compare against the Results table; character-width checks (M3) must count characters, not bytes, since every glyph in the chrome is multi-byte UTF-8.

## Session liveness (`STATUS`, `detail::sessions` tags, `live::`) — manual session (2026-08-23)

Ran against the real local store (914 projects, 39 of them live at the time), on the host rather than inside the container — the feature reads `/proc` and `~/.claude/history.jsonl`, neither of which the container sees the way the operator does. 22 cases; the four defects found were fixed and covered before this record was written.

### Why manual testing was needed here

This is the first feature in the crate whose inputs a fixture cannot construct. Liveness is *inferred* from the process table and the prompt history, and no test can conjure an attached Claude Code process whose cwd is a freshly-created temp directory. The integration tests therefore assert only the negative half of the contract (a fixture is never live, so every affordance must be absent), and `src/cli/liveness.rs`'s unit tests assert the positive half against a synthetic `/proc`-shaped tree. Neither can answer the questions that decided whether the feature was right: does the history join actually pin one session per live project across 39 of them, does a real store produce the state mix the renderer assumes, and do the two `detail::sessions` renderers agree. Three of the four defects below are invisible to both test layers.

### Results

| # | Case | Result |
|---|------|--------|
| M1 | `STATUS` column present with live rows, absent without | ✅ conditional, same convention as `⚠ gone` |
| M2 | Column alignment with `STATUS` inserted | ✅ consistent across flat and `live::1` views |
| M3 | Session tag under `detail::sessions`, flat layout | ✅ `● working` on the driven conversation |
| M4 | Session tag under `detail::sessions`, tree layout | ⚠️ **defect found** — see L1; ✅ after fix |
| M5 | One tag per live project across the whole store | ✅ 39/39 tagged exactly one session — 0 untagged, 0 multiply-tagged |
| M6 | `ids::1 live::1` on a live project | ⚠️ **defect found** — see L2; ✅ after fix |
| M7 | `ids::1 live::0` on a live project | ⚠️ same defect; ✅ suppresses correctly after fix |
| M8 | `ids::1 count::1` with each `live::` value | ✅ counts agree with the id lines after fix |
| M9 | Summary line with a mixed live set | ✅ `39 live (6 working, 33 waiting)` |
| M10 | Summary line with a single-state live set | ⚠️ **defect found** — see L3; ✅ `1 live (working)` after fix |
| M11 | Summary line with nothing live | ✅ clause absent entirely, never `0 live` |
| M12 | Structural tree nodes (ancestors owning no session) | ✅ blank `STATUS` — a node never inherits a child's attachment |
| M13 | `▸` cwd gutter alongside `STATUS` | ✅ both render, no collision |
| M14 | Project-level vs session-level state agreement | ✅ consistent within one invocation; apparent drift across two invocations is the clock, not the logic |
| M15 | Orphan families (no root session) and tag placement | ✅ orphans sort to the tail (`UNIX_EPOCH` key), never displacing a real root's rank |
| M16 | `live::` with `filter::` | ✅ intersects rather than short-circuiting |
| M17 | `live::` with `agent::0` / `agent::1` | ✅ agent sidecars never carry a tag under either |
| M18 | `live::` with `limit::` / `show_topic::` | ✅ tag renders before the topic text |
| M19 | `live::bogus`, `live::2` | ✅ exit 1, argument error before any storage access |
| M20 | `.list live::1` | ✅ rejected — the parameter is registered on `.projects` only |
| M21 | Help text | ✅ `live::` listed with both documented examples |
| M22 | Future mtime on an attached project | ⚠️ **defect found** — see L4; ✅ after fix |

### Defects found and fixed

| # | Symptom | Root cause | Fix | Regression test |
|---|---------|------------|-----|-----------------|
| L1 | `detail::sessions show_tree::1` showed no state tag, while the flat layout showed `● working` for the same conversation at the same moment | `render_families_v2` was never passed the `LivenessMap` — only the flat renderer received it. Choosing a layout silently decided whether "is this one running" got answered at all. | Thread the map into `render_families_v2` and tag the root through the same `session_liveness` call the flat path uses | `cli_param_live_test.rs::ec_11_tree_and_flat_layouts_agree_on_the_state_tag` (pins the two paths to each other; the positive direction is unreachable from a fixture) |
| L2 | `ids::1 live::1` and `ids::1 live::0` produced byte-identical output on a project that *was* live — one of them had to be wrong | The `ids::` branch returns at step 1, long before the listing path computes liveness at step 6, so `live::` was read by nobody. Worst case for a scripting mode: the caller has no rendered output to notice the discrepancy in. | Apply `live::` in the branch as a predicate on the named project; probe only when the parameter is set, so plain `ids::1` still costs no process-table walk. Unavailable detection exits non-zero with the reason on stderr rather than emitting an empty list a script would read as fact. | `cli_param_live_test.rs::ec_9_…` / `ec_10_…` |
| L3 | Summary line rendered `1 live (1 working, 0 waiting)` and `6 live (0 working, 6 waiting)` | The live clause always spelled out both halves, contradicting the same line's own convention — asserted by OV-4 — that a zero `agents` segment is omitted entirely. | Collapse the breakdown to the single state's name when one half is zero: `1 live (working)` | `projects_overview.rs::overview_tests::test_live_clause_collapses_a_zero_half` |
| L4 | An mtime ahead of the local clock classified as `waiting` — the freshest possible write reported as the least active | `SystemTime::now().duration_since( mtime )` returns `Err` for a future timestamp, and `is_ok_and` folds that error in with "too old". Reachable through ordinary clock skew against an NFS or container host, not just a deliberate `touch -d`. | Match on the result explicitly; `Err` means "at least as recent as now", so it is `Working` | `liveness.rs::liveness_tests::test_future_mtime_is_working_not_waiting` |

One further defect surfaced during verification rather than from the feature itself: `projects_edge_case_test.rs`'s `ec5`/`ec7`/`ec8` byte-compared stdout from two separate spawns of an age-rendering command, so a run that straddled a second boundary failed on `1s ago` vs `3s ago`. It reproduced 2/2 under full-suite load and passed 3/3 in isolation. `cli_cmd_projects_test.rs` already carried a private `normalize_relative_time` for exactly this, so the fix was to promote that helper to `tests/common/mod.rs`, widen it to also absorb the column padding the age token drives, and route all three sites through it. 12 stress iterations against a concurrently-running full suite now pass 12/12.

### Reproducing

The battery is a disposable script, not a committed fixture — it reads the operator's own process table and prompt history, so its expected values are local and change minute to minute. To rebuild it, pin a built `clg` outside `target/` first: a concurrent `RUSTFLAGS`-differing cargo invocation changes the fingerprint and evicts the binary mid-run, which presents as `No such file or directory` from the middle of a case list. Run on the host, not in the container — inside it `/proc` lists the container's processes and every project reads as not-attached, which is the documented "detection never claims a negative" case rather than a bug. M5 is the case worth rebuilding first: it is the only one that exercises the history join at real scale, and it is what would catch a regression that tags every session in a live project instead of one.
