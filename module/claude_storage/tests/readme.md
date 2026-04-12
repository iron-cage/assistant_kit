# Test Suite Organization

## Overview

The claude_storage test suite uses automated tests for parameter validation and integration testing, with manual tests for exploratory and user experience validation.

## Test Structure

```
tests/
├── readme.md                              # This file - test suite organization
├── common/                                 # Shared test utilities
│   └── mod.rs                             # Pre-compiled binary helper (cargo_bin!)
├── behavior/                               # Behavior hypothesis invalidation tests (B1..B11)
│   ├── mod.rs                             # Test binary root; shared helpers for real ~/.claude/ inspection
│   ├── b01_default_continues.rs           # B1 — default invocation continues most recent session
│   ├── b02_new_session.rs                 # B2 — --new-session creates separate .jsonl
│   ├── b03_print_flag.rs                  # B3 — -p is output mode, not session flag
│   ├── b04_continue_flag.rs              # B4 — -c aliases default continuation
│   ├── b05_mtime_selection.rs             # B5 — current session selected by mtime
│   ├── b06_session_accumulation.rs        # B6 — sessions accumulate as separate files
│   ├── b07_agent_sessions.rs              # B7 — agent sessions are agent-*.jsonl siblings
│   ├── b08_zero_byte_init.rs              # B8 — 0-byte .jsonl created as placeholder on startup
│   ├── b09_storage_path.rs                # B9 — project path uses /→- encoding
│   ├── b10_entry_threading.rs             # B10 — entries linked via parentUuid
│   └── b11_auto_continue.rs              # B11 — CLAUDE_CODE_AUTO_CONTINUE env var
├── manual/                                 # Manual testing plans and results
│   └── readme.md                          # Manual testing plan for this crate
├── cli_commands.rs                        # CLI command storage operations
├── cli_sanity.rs                          # CLI binary build and run verification
├── command_version_consistency_test.rs    # Command version consistency tests
├── content_display_integration_test.rs    # Content display behavior tests
├── count_command_bug_fix.rs               # .count context-awareness bug fix (Bug #003)
├── export_command_test.rs                 # .export parameter validation tests (Phase 1C)
├── lib_test.rs                            # Library API smoke tests
├── list_command_test.rs                   # .list parameter bounds and combinations
├── list_smart_session_display.rs          # .list smart session display tests
├── parameter_validation_test.rs           # Multi-command parameter validation tests
├── path_resolution_integration_test.rs    # Path resolution tests
├── path_resolution_test.rs                # Path resolution unit tests
├── project_parameter_bug_fix.rs           # Project parameter parsing tests
├── project_parameter_multi_command_bug.rs # Project parameter across commands (#012)
├── project_parameter_relative_path_bug.rs # Relative path resolution (#013)
├── search_command_test.rs                 # .search parameter validation tests (Phase 1B)
├── search_session_partial_uuid_bug.rs     # .search session partial UUID fix (issue-020)
├── search_special_characters_bug.rs       # Special character handling (Bug #006, #007)
├── session_path_command_test.rs           # .path/.exists/.session.dir/.session.ensure lifecycle commands
├── projects_command_test.rs               # .projects scope-aware listing and parameter validation (issue-024/029/031/032 regression)
├── projects_output_format_test.rs         # .projects output format redesign (plan-004): IT-17 through IT-22
├── smart_show_command.rs                  # .show smart parameter detection tests
├── status_path_test.rs                    # .status path parameter tests (Phase 1D)
└── truncate_utf8_bug.rs                   # Truncation safety on multibyte UTF-8 (issue-018)
```

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib_test.rs` | Library API: `COMMANDS_YAML` exists, `register_commands()` callable |
| `common/mod.rs` | Pre-compiled binary helper for integration tests |
| `behavior/mod.rs` | Behavior test binary root; shared helpers for real `~/.claude/` inspection |
| `behavior/b01_default_continues.rs` | B1: resumable session exists in real storage |
| `behavior/b02_new_session.rs` | B2: `--new-session` documented in `claude --help` |
| `behavior/b03_print_flag.rs` | B3: `-p`/`--print` flag documented in `claude --help` |
| `behavior/b04_continue_flag.rs` | B4: `-c`/`--continue` flag documented in `claude --help` |
| `behavior/b05_mtime_selection.rs` | B5: multiple sessions have distinct observable mtimes |
| `behavior/b06_session_accumulation.rs` | B6: sessions accumulate as separate `.jsonl` files |
| `behavior/b07_agent_sessions.rs` | B7: agent sessions are `agent-*.jsonl` siblings with `isSidechain:true` |
| `behavior/b08_zero_byte_init.rs` | B8: 0-byte `.jsonl` placeholder files exist in real storage |
| `behavior/b09_storage_path.rs` | B9: project dir names follow `/`→`-` encoding convention |
| `behavior/b10_entry_threading.rs` | B10: conversation entries linked via `parentUuid` (null root, non-null chain) |
| `behavior/b11_auto_continue.rs` | B11: `CLAUDE_CODE_AUTO_CONTINUE` env var recognized by `claude` |
| `cli_commands.rs` | Test CLI command storage operations |
| `cli_sanity.rs` | Verify CLI binary builds and runs |
| `command_version_consistency_test.rs` | Validate version annotation consistency |
| `content_display_integration_test.rs` | Test content-first display (REQ-011) |
| `count_command_bug_fix.rs` | Test .count context-awareness and path projects |
| `export_command_test.rs` | Validate .export command parameters |
| `list_command_test.rs` | Validate .list command parameter bounds and combinations |
| `list_smart_session_display.rs` | Test smart session display in .list |
| `parameter_validation_test.rs` | Validate CLI parameter handling |
| `path_resolution_integration_test.rs` | Test path resolution in .list command |
| `path_resolution_test.rs` | Test path:: parameter smart detection |
| `project_parameter_bug_fix.rs` | Test project parameter ID resolution |
| `project_parameter_multi_command_bug.rs` | Test project parameter across commands |
| `project_parameter_relative_path_bug.rs` | Test relative path resolution (Finding #013) |
| `search_command_test.rs` | Validate .search command parameters |
| `search_session_partial_uuid_bug.rs` | Test partial UUID matching in .search session filter |
| `search_special_characters_bug.rs` | Test special character handling in queries |
| `session_path_command_test.rs` | Test .path/.exists/.session.dir/.session.ensure lifecycle commands |
| `projects_command_test.rs` | Test .projects scope-aware session listing and parameter validation |
| `projects_output_format_test.rs` | Test .projects output format redesign (plan-004): path headers, agent collapse, entry counts |
| `smart_show_command.rs` | Test location-aware .show command |
| `status_path_test.rs` | Test path parameter in .status command |
| `truncate_utf8_bug.rs` | Test truncation safety on multibyte UTF-8 (issue-018) |

## Test Documentation Standards

### Feature Tests (New Commands/Parameters)

Use 4-section Purpose format:

```rust
/// Test {command} {parameter} {validation_type}
///
/// ## Purpose
/// {What this test validates and why it matters}
///
/// ## Coverage
/// {Specific corner case or requirement being tested}
///
/// ## Validation Strategy
/// {How the test verifies behavior - assertions used}
///
/// ## Related Requirements
/// {REQ-NNN or doc instance (docs/feature/NNN_name.md) that this test validates}
#[test]
fn test_{command}_{parameter}_{case}()
```

**Examples**:
- `tests/search_command_test.rs::test_search_query_required`
- `tests/export_command_test.rs::test_export_session_id_required`
- `tests/status_path_test.rs::test_status_custom_path`

### Bug Fix Tests (Finding #NNN)

Use 5-section Root Cause format with Fix comment in source:

```rust
/// Test {command} {parameter} {issue} (Finding #NNN)
///
/// ## Root Cause
/// {Technical explanation of why bug occurred}
///
/// ## Why Not Caught
/// {Gap in existing tests that allowed bug}
///
/// ## Fix Applied
/// {What validation was added}
///
/// ## Prevention
/// {Policy to prevent similar bugs}
///
/// ## Pitfall
/// {Anti-pattern that caused bug}
#[test]
fn test_{command}_{parameter}_{issue}()
```

**Source Code Fix Comment** (3 required fields):
```rust
// Fix(issue-NNN): {One-line description}
//
// Root cause: {Why bug occurred}
//
// Pitfall: {Anti-pattern to avoid}
```

**Example**:
- Test: `tests/search_command_test.rs::test_search_verbosity_invalid`
- Fix comment: `src/cli/mod.rs:1183-1200` (Finding #010)

## Integration Test Strategy

Tests that depend on real storage state or external resources should be marked `#[ignore]`:

```rust
#[test]
#[ignore = "Integration test - depends on actual ~/.claude/ storage state"]
fn test_status_default_path()
```

**Why**:
- Prevents test failures due to environmental factors (corrupted sessions, missing directories)
- Allows tests to be run selectively with `cargo test -- --ignored`
- Separates unit/validation tests from integration tests

**Examples**:
- `tests/status_path_test.rs::test_status_default_path` - depends on ~/.claude/ state
- `tests/search_command_test.rs::test_search_entry_type_valid` - requires real session data
- `tests/export_command_test.rs::test_export_format_valid` - requires real session data

## Test Naming Conventions

```
test_{command}_{parameter}_{scenario}
```

**Examples**:
- `test_search_query_required` - .search command, query parameter, required validation
- `test_export_format_invalid` - .export command, format parameter, invalid value rejection
- `test_status_path_with_verbosity` - .status command, path+verbosity parameters, interaction

## Test Organization Principles

### Command-Specific Files

Each command gets its own test file for parameter validation:
- `search_command_test.rs` - .search parameter validation
- `export_command_test.rs` - .export parameter validation
- `status_path_test.rs` - .status path parameter tests

### Shared Validation Files

Cross-command tests in shared files:
- `parameter_validation_test.rs` - Multi-command parameter validation tests

### Integration Test Files

Feature-specific integration tests:
- `content_display_integration_test.rs` - Content display behavior
- `list_smart_session_display.rs` - Smart session display auto-enable
- `path_resolution_integration_test.rs` - Path resolution with real filesystem

## Test Quality Standards

### Documentation Quality

Test documentation must be:
- **Specific**: Technical details, not generic statements ("Fixed bug" → "search_routine missing verbosity validation")
- **Actionable**: Clear prevention steps ("Don't assume defaults prevent invalid input")
- **Traceable**: Links to requirements (REQ-012), issues (Finding #010), source locations
- **Concise**: Essential information only, no redundancy

### Test Coverage

All parameters must have validation tests:
- Required parameters → test missing parameter error
- Optional parameters → test default value behavior
- Enumerated values → test invalid value rejection
- Ranges → test boundary values and out-of-range rejection
- Booleans → test invalid value rejection (not 0 or 1)

### No Mocking

Tests must use real implementations or be marked `#[ignore]`:
- ✅ Use `TempDir` for real filesystem operations
- ✅ Mark tests requiring real storage as `#[ignore]`
- ❌ Don't mock Storage, Command, or core functionality

## Test Execution Architecture

Integration tests use a pre-compiled binary helper (`common::claude_storage_cmd()`)
instead of `cargo run` to avoid compilation during test execution.

**Why**: Each `cargo run` inside a test triggers a full cargo compilation cycle
(300s+). Under workspace-wide nextest runs, this exceeds the 300s timeout.

**Fix**: `assert_cmd::cargo::cargo_bin!("claude_storage")` resolves to the binary
path built by nextest BEFORE test execution. No recompilation at test time.

**Pattern**: All test files declare `mod common;` and use `common::claude_storage_cmd()`
instead of `Command::new("cargo").args(["run", ...])`.

**Test Isolation with `CLAUDE_STORAGE_ROOT`**:

Tests that write fixture data use the `CLAUDE_STORAGE_ROOT` env var to redirect storage
to a `TempDir`, so they never touch real `~/.claude/` state and run safely in parallel:

```rust
let dir = tempfile::TempDir::new().unwrap();
// write fixture data under dir.path()...
let output = common::claude_storage_cmd()
  .env("CLAUDE_STORAGE_ROOT", dir.path())
  .args([".list"])
  .output()
  .unwrap();
```

Set the env var on the **subprocess** (`cmd.env(…)`), NOT via `std::env::set_var()`,
which is process-wide and causes nextest parallel-test race conditions.

## Test Verification Commands

```bash
# Run all effective tests (excludes ignored tests)
w3 .test l::3           # Default (recommended)
ctest3                  # Alias for w3 .test l::3

# Run specific test file
cargo nextest run --test search_command_test --all-features

# Run ignored tests only
cargo nextest run --all-features -- --ignored

# Run all tests including ignored
cargo nextest run --all-features -- --include-ignored
```

## Test Count Tracking

**Current Status**: 242 tests, 0 ignored
- Effective tests: 242 (all tests run fully)
- Ignored tests: 0 (target met — all tests use `CLAUDE_STORAGE_ROOT` + `TempDir` isolation)

## Known Findings

### Finding #009: .count target parameter validation
- **Issue**: Missing validation for target parameter (accepted invalid values)
- **Tests**: 4 tests added in `parameter_validation_test.rs`
- **Fix**: Added validation at `src/cli/mod.rs:1151-1157`
- **Documentation**: Fix(issue-009) comment in source

### Finding #010: .search verbosity parameter validation
- **Issue**: search_routine missing verbosity range validation (0-5), inconsistent with other commands
- **Test**: `test_search_verbosity_invalid` in `search_command_test.rs`
- **Fix**: Added validation at `src/cli/mod.rs:1183-1200`
- **Root Cause**: Assumed default values prevent invalid input (they don't)
- **Documentation**: Fix(issue-010) comment in source + 5-section test documentation

### Finding #013: Relative path resolution in project parameter
- **Issue**: parse_project_parameter does not resolve ".", "..", "~" as paths
- **Tests**: 4 tests in `project_parameter_relative_path_bug.rs`
- **Fix**: Added relative path detection before UUID default case
- **Root Cause**: Only handled absolute paths, path-encoded, and UUID; missed relative paths
- **Documentation**: Fix(issue-013) comment in source + 5-section test documentation

### Finding #014: Path resolution in status_routine
- **Issue**: status_routine does not resolve ".", "..", "~" in path parameter
- **Tests**: 2 tests in `status_path_test.rs` (test_status_path_dot_resolves_to_cwd, test_status_path_tilde_resolves_to_home)
- **Fix**: Added resolve_path_parameter() call before Storage::with_root()
- **Root Cause**: status_routine passed path directly without resolving, unlike list_routine
- **Documentation**: Fix(issue-014) comment in source + 5-section test documentation

### Finding #015: list_routine missing verbosity range validation ✅ Fixed
- **Issue**: `list_routine` did not validate verbosity 0-5 range; `-1` or `6` were silently accepted
- **Tests**: 5 verbosity tests in `list_command_test.rs` (N: -1, 6; P: 0, 3, 5)
- **Fix**: Added `if !(0..=5).contains(&verbosity)` check in `list_routine` after get_integer call
- **Root Cause**: Verbosity extracted without bounds check, unlike `status_routine` and `search_routine`
- **Documentation**: Fix(issue-015) comment in `src/cli/mod.rs` + 5-section test documentation

### Finding #016: show_project_routine missing verbosity range validation ✅ Fixed (command removed in task-013)
- **Issue**: `show_project_routine` did not validate verbosity 0-5 range; same gap as Finding #015
- **Tests**: 4 verbosity tests existed — test file deleted with command removal (task-013)
- **Fix**: Added `if !(0..=5).contains(&verbosity)` check in `show_project_routine` after get_integer call
- **Root Cause**: Verbosity passed unvalidated to impl functions; invalid values produced garbled output
- **Note**: `.show.project` command removed in task-013; pattern applies to any routine that accepts verbosity

### issue-015: .status performance — global_stats() O(total JSONL bytes)
- **Issue**: `.status` took >2 minutes with 1903 projects / 7 GB JSONL
- **Tests**: `status_global_stats_fast_bug.rs` in `claude_storage_core/tests/`
- **Fix**: Added `global_stats_fast()` (filesystem-only); `status_routine` uses it for verbosity 0-1
- **Root Cause**: `global_stats()` parsed all session JSONL to count entries/tokens — O(total JSONL bytes)
- **Documentation**: Fix(issue-015) in `storage.rs` + `status_global_stats_fast_bug.rs`

### issue-016: count_entries() counted all JSONL lines, not conversation entries
- **Issue**: `.count target::entries` returned 2135 while `.show` "Total Entries" showed 2034 (101 discrepancy)
- **Tests**: `count_entries_bug.rs` in `claude_storage_core/tests/`
- **Fix**: Changed `count_entries()` to parse `"type"` field and count only `"user"`/`"assistant"` entries
- **Root Cause**: `content.lines().count()` counted every non-empty JSONL line including metadata
- **Documentation**: Fix(issue-016) in `session.rs` + `count_entries_bug.rs`

### issue-017: .count crashes on IO errors in sessions instead of skipping with warning
- **Issue**: `.count` from a project with any session causing an IO error (e.g., unreadable file) failed entirely (exit 1)
- **Test**: `test_count_skips_unreadable_sessions` in `count_command_bug_fix.rs`
- **Fix**: Changed `?` to `match` + `eprintln!` warning in context-aware loop in `count_routine()`
- **Root Cause**: `?` propagated `count_entries()` IO errors from individual sessions to entire command
- **Note**: Truncated JSONL does NOT trigger this — `count_entries()` uses byte-level search and succeeds on partial lines; only IO errors (e.g., permission denied) cause failure
- **Documentation**: Fix(issue-017) in `cli/mod.rs` + 5-section test doc in `count_command_bug_fix.rs`

### issue-018: `truncate_if_needed` panics on multibyte UTF-8 text
- **Issue**: `&text[..len]` slices by byte offset, panicking when `len` lands inside a multibyte UTF-8 sequence (emoji, CJK, accented characters)
- **Tests**: 7 tests in `truncate_utf8_bug.rs` (tc001-tc007: emoji, CJK, boundary, zero-length)
- **Fix**: Walk backwards from `len` using `is_char_boundary()` to find nearest valid boundary
- **Root Cause**: `str::len()` returns bytes, not characters; using it directly as a slice bound on user-supplied text panics on non-ASCII content
- **Documentation**: Fix(issue-018) in `cli/mod.rs` + 5-section test doc in `truncate_utf8_bug.rs`

### issue-025: Singular/plural mismatch in "Found N X:" output headers
- **Issue**: `.list`, `.search`, and `.projects` all output "Found 1 projects:", "Found 1 matches:", "Found 1 sessions:" — incorrect plural when count == 1
- **Tests**: 7 tests across 3 files (IT-14..IT-16 in `projects_command_test.rs`; 2 in `list_command_test.rs`; 2 in `search_command_test.rs`)
- **Fix**: Derive noun ("project"/"projects", "match"/"matches", "session"/"sessions") from count before formatting header; zero uses plural
- **Root Cause**: `writeln!(output, "Found {} noun:\n", count)` used a hardcoded plural noun string regardless of count
- **Documentation**: 5-section doc block at issue-025 comment in each test file; source changes are minimal inline fixes

### issue-027: list_routine verbosity 1 per-project session count uses wrong plural
- **Issue**: `.list sessions::1` showed `Uuid("proj") (1 sessions)` — should be `(1 session)` (singular)
- **Tests**: `test_list_session_count_singular_when_one_session`, `test_list_session_count_plural_when_multiple_sessions` in `list_command_test.rs`
- **Fix**: Derive `noun` from `session_count` before format string, same pattern as issue-025 header fix
- **Root Cause**: `writeln!(output, "{:?} ({} sessions)", ...)` used hardcoded plural — sibling of the issue-025 bug in a different format string in the same routine
- **Documentation**: Fix(issue-027) in `cli/mod.rs` + 5-section test doc in `list_command_test.rs`

### issue-026: export_session_to_file uses bare `?` losing path context in IO errors
- **Issue**: `.export output::/nonexistent/dir/file.md` produced "I/O error during unknown operation: No such file or directory" with no indication of which path failed
- **Test**: `test_export_output_path_in_error_message` in `export_command_test.rs`
- **Fix**: Changed `File::create(output_path)?` to `.map_err(|e| Error::io(e, format!("create output file '{}'", ...)))` in `export_session_to_file`
- **Root Cause**: Blanket `From<io::Error> for Error` always sets context to "unknown operation". Any `?` on an IO operation silently loses path/operation context.
- **Documentation**: Fix(issue-026) in `export.rs` + 5-section test doc in `export_command_test.rs`

### plan-004: projects_routine output format redesign

- **Issue**: `.projects` output was a flat list of session IDs with opaque encoded project labels (e.g. `"-home-user1-pro"`); no project grouping, no readable paths, no agent collapse at scale
- **Tests**: 6 tests IT-17..IT-22 in `projects_output_format_test.rs` (IT-23 covers display fix issue-029)
- **Fix**: Redesigned `projects_routine` to group sessions by `BTreeMap<String, Vec<Session>>` keyed by decoded project path; added `decode_project_display()` helper; agent sessions collapsed at v1 with no `agent::` filter; entry counts shown per session at v2+; blank line between project groups
- **Root Cause**: Original design used flat `Vec<(label, id)>` with labels from `format!("{:?}", project.id())` — debug-format encoded strings, not human-readable paths
- **Pitfalls**:
  1. `decode_path()` requires input starting with `-`; UUID project dirs don't → must guard with `starts_with('-')` before calling decode
  2. Topic suffix `--topic` must be stripped (`find("--")`) before calling `decode_path`; otherwise it becomes a phantom path component
  3. Blank line between project groups was not in the initial implementation despite being in the design algorithm and the docs example — always verify format output against docs examples
- **Note**: Originally tagged as issue-026 internally; relabeled to plan-004 because issue-026 was already assigned to the export path-in-error-message bug

### issue-029: decode_project_display splits underscore dirs as path separators

- **Issue**: `.sessions scope::under` (and all verbosity ≥ 1 scopes) displayed project path headers with underscore-named directories split on `/` — e.g., `~/wip_core/myproject:` shown as `~/wip/core/myproject:`
- **Test**: `IT-23` (`test_sessions_under_display_preserves_underscores`) in `projects_command_test.rs`; marked `bug_reproducer(issue-029)`
- **Fix**: Added `decode_path_via_fs()` + `walk_fs()` in `cli/mod.rs`; `decode_project_display` now tries the heuristic result first — if it doesn't exist on disk, falls back to FS-guided DFS that chooses `/` vs `_` at each boundary by calling `is_dir()` on candidate prefixes; final fallback is the heuristic result (handles deleted/remote projects)
- **Root Cause**: `encode_path` maps both `_` (underscore) and `/` (path separator) to `-`; `decode_component` heuristic defaulted to `/` for all unrecognized `-` boundaries, so `wip-core` always decoded to `wip/core` regardless of whether a real `wip_core` directory exists
- **Documentation**: Fix(issue-029) + 3-field source comment in `cli/mod.rs`; 5-section test doc block in `projects_command_test.rs`

### issue-031: scope::under includes sibling modules with underscore-suffix names

- **Issue**: `scope::under path::claude_storage` incorrectly included sessions from `claude_storage_core` — a sibling module at the same directory level
- **Test**: `IT-25` (`it_25_scope_under_excludes_underscore_named_sibling`) in `projects_command_test.rs`; marked `bug_reproducer(issue-031)`
- **Fix**: Two-stage predicate in the `"under"` arm of `project_matches` in `projects_routine`. String prefix is fast-reject only; `decode_path_via_fs` + `Path::starts_with` (component-wise) verifies ambiguous candidates. `--topic` suffix stripped before filesystem walk.
- **Root Cause**: `encode_path` maps both `_` and `/` to `-`; string `starts_with` on encoded forms cannot distinguish `base/sub` (encoded `base-sub`) from `base_extra` (encoded `base-extra`) — both share the `base-` prefix
- **Documentation**: Fix(issue-031) + 3-field source comment in `cli/mod.rs`; 5-section test doc block in `projects_command_test.rs`

### issue-032: scope::relevant includes sibling projects with underscore-suffix names

- **Issue**: `scope::relevant path::base_extra` incorrectly included sessions from the sibling project `base` — not an ancestor of `base_extra` despite passing the string prefix check
- **Test**: `IT-26` (`it_26_scope_relevant_excludes_underscore_named_sibling`) in `projects_command_test.rs`; marked `bug_reproducer(issue-032)`
- **Fix**: Two-stage predicate in the `"relevant"` arm of `project_matches`. `is_relevant_encoded` is fast-reject only; `decode_path_via_fs` + `base_path.starts_with(decoded_path)` (component-wise) verifies prefix-match candidates.
- **Root Cause**: `is_relevant_encoded` used `encoded_base.starts_with(dir_name + "-")` which cannot distinguish ancestor `base` (child path `base/sub` → `base-sub`) from sibling `base` (when base_path is `base_extra` → `base-extra`) — same underscore/slash encoding ambiguity as issue-031
- **Documentation**: Fix(issue-032) + 3-field source comment in `cli/mod.rs`; 5-section test doc block in `projects_command_test.rs`

### issue-033: `.exists` stderr output violated spec ("no sessions" vs multi-level wrapped error)

- **Issue**: `execute_oneshot` printed `"Error: Execution error: Execution Error: no sessions"` but the spec requires exactly `"no sessions"` on stderr for exit-1 case
- **Test**: `it_exists_stderr_exact_when_no_history` in `session_path_command_test.rs`; marked `bug_reproducer(issue-033)`
- **Fix**: Added `extract_user_message()` in `cli_main.rs` that strips `"Execution error: Execution Error: "` prefix before printing in one-shot mode
- **Root Cause**: `execute_oneshot` used `eprintln!("Error: {error}")` where the unilang pipeline had already double-wrapped the message. `ErrorData::Display` uses `writeln!` (adds `\n`) → `Error::Execution` adds `"Execution Error: "` → pipeline adds `"Execution error: "` → `execute_oneshot` adds `"Error: "` = four layers
- **Documentation**: Fix(issue-033) comment in `cli/mod.rs` exists_routine + 5-section test doc in `session_path_command_test.rs`

### issue-028: "1 entries" — hardcoded plural "entries" in session header and project session list
- **Issue**: (a) `.show session_id::abc` produced "Session: abc (1 entries)" — wrong plural in header; (b) `.show.project verbosity::1` with 1-entry session showed "(1 entries, last: ...)" — same root cause
- **Tests**: `test_show_session_single_entry_header_says_entry_not_entries`, `test_show_session_multi_entry_header_still_says_entries` in `smart_show_command.rs`; `.show.project` tests deleted with command removal (task-013)
- **Fix**: Added `entry_noun`/`e_noun` variables derived from count (1 → "entry", else "entries") in `show_session_routine` in `cli/mod.rs`; same fix was in `show_project_routine` (now removed)
- **Root Cause**: Format strings hardcoded "entries" regardless of count — same pattern as issue-025/027 but for the irregular noun "entry"/"entries"
- **Documentation**: Fix(issue-028) in `cli/mod.rs` (two locations) + 5-section test doc in both test files

## Manual Testing

See `tests/manual/readme.md` for manual testing plan and procedures.

## Related Documentation

- **Documentation**: `docs/entities.md` - Command specifications and behavioral requirements index
- **Code Design**: See applicable rulebooks via `clm .rulebooks.list`
- **Test Organization**: `test_organization.rulebook.md` - Test documentation format standards
- **Codebase Hygiene**: `codebase_hygiene.rulebook.md` - Quality standards for documentation
