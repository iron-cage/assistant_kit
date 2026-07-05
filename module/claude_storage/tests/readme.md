# Test Suite Organization

## Overview

The claude_storage test suite uses automated tests for parameter validation and integration testing, with manual tests for exploratory and user experience validation.

## Test Structure

```
tests/
├── readme.md                              # This file - test suite organization
├── common/                                 # Shared test utilities
│   └── mod.rs                             # Pre-compiled binary helper (cargo_bin!)
├── docs/                                   # Test documentation mirroring docs/ hierarchy
│   └── cli/                               # CLI test case indexes (command, param, param_group)
├── manual/                                 # Manual testing plans and results
│   └── readme.md                          # Manual testing plan for this crate
├── cli_commands.rs                        # CLI command storage operations
├── cli_sanity.rs                          # CLI binary build and run verification
├── command_version_consistency_test.rs    # Command version consistency tests
├── content_display_integration_test.rs    # Content display behavior tests
├── count_command_bug_fix.rs               # .count context-awareness bug fix (Bug #003)
├── count_command_test.rs                  # .count target::conversations tests (IT-T04..IT-T05)
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
├── session_path_command_test.rs           # .project.path/.project.exists/.session.dir/.session.ensure lifecycle commands
├── projects_command_test.rs               # .projects filter/validation/output formatting (show_tree, session/agent/min_entries filters, IT-14..IT-16, IT-50)
├── projects_edge_case_test.rs             # .projects scope parameter acceptance/rejection (EC-1..EC-9)
├── projects_scope_test.rs                 # .projects scope behavioral semantics: local/under/relevant/global, underscore paths (IT-9..IT-13), topic dirs
├── projects_family_display_test.rs        # .projects family/agent session display (IT-1, IT-33, IT-36..IT-48)
├── projects_path_encoding_test.rs         # .projects path decode/display bug reproducers (IT-23..IT-26, IT-60..IT-64)
├── projects_output_format_test.rs         # .projects output format: path headers, agent collapse (IT-17..IT-22); list-mode redesign (IT-52..IT-53)
├── projects_scope_around_test.rs          # .projects scope::around bidirectional neighborhood semantics (IT-57..IT-59)
├── projects_zero_byte_count_bug.rs        # .projects zero-byte session exclusion from header count (issue-034, IT-54..IT-56)
├── smart_show_command.rs                  # .show smart parameter detection tests
├── status_path_test.rs                    # .status path parameter tests (Phase 1D)
├── truncate_utf8_bug.rs                   # Truncation safety on multibyte UTF-8 (issue-018)
├── feature_cli_tool_test.rs               # Feature tests for the CLI tool (FT-1..FT-6)
└── operation_migration_guide_test.rs      # Migration guide procedure tests (OP-1..OP-5)
```

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `docs/` | Test documentation parallel to `docs/` (test case indexes, test plans) |
| `lib_test.rs` | Library API: `COMMANDS_YAML` exists, `register_commands()` callable |
| `common/mod.rs` | Pre-compiled binary helper for integration tests |
| `cli_commands.rs` | Test CLI command storage operations |
| `cli_sanity.rs` | Verify CLI binary builds and runs |
| `command_version_consistency_test.rs` | Validate version annotation consistency |
| `content_display_integration_test.rs` | Test content-first display (REQ-011) |
| `count_command_bug_fix.rs` | Test .count context-awareness and path projects |
| `count_command_test.rs` | Test .count target::conversations (IT-T04..IT-T05) |
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
| `session_path_command_test.rs` | Test .project.path/.project.exists/.session.dir/.session.ensure lifecycle commands |
| `projects_command_test.rs` | Test .projects filter/validation/output formatting (show_tree, session/agent/min_entries filters, IT-14..IT-16, IT-50) |
| `projects_edge_case_test.rs` | Test .projects scope parameter acceptance/rejection edge cases (EC-1..EC-9) |
| `projects_scope_test.rs` | Test .projects scope behavioral semantics: local/under/relevant/global, underscore paths (IT-9..IT-13), topic dirs |
| `projects_family_display_test.rs` | Test .projects family and agent session display (IT-1, IT-33, IT-36..IT-48) |
| `projects_path_encoding_test.rs` | Test .projects path decode/display bug reproducers (IT-23..IT-26, IT-60..IT-64) |
| `projects_output_format_test.rs` | Test .projects output format: path headers, agent collapse (IT-17..22); list-mode redesign (IT-52..53) |
| `projects_scope_around_test.rs` | Test .projects scope::around bidirectional neighborhood semantics (IT-57..IT-59) |
| `projects_zero_byte_count_bug.rs` | Test zero-byte session exclusion from .projects list-mode header count (issue-034) |
| `smart_show_command.rs` | Test location-aware .show command |
| `status_path_test.rs` | Test path parameter in .status command |
| `truncate_utf8_bug.rs` | Test truncation safety on multibyte UTF-8 (issue-018) |
| `feature_cli_tool_test.rs` | Test feature-level CLI tool cases (FT-1..FT-6) |
| `operation_migration_guide_test.rs` | Test migration guide procedure cases (OP-1..OP-5) |
| `cli_cmd_status_test.rs` | Spec-driven integration tests for `.status` command |
| `cli_cmd_list_test.rs` | Spec-driven integration tests for `.list` command |
| `cli_cmd_show_test.rs` | Spec-driven integration tests for `.show` command |
| `cli_cmd_count_test.rs` | Spec-driven integration tests for `.count` command |
| `cli_cmd_search_test.rs` | Spec-driven integration tests for `.search` command |
| `cli_cmd_export_test.rs` | Spec-driven integration tests for `.export` command |
| `cli_cmd_projects_test.rs` | Spec-driven integration tests for `.projects` command (INT-1..INT-25) |
| `cli_cmd_projects_summary_test.rs` | Spec-driven integration tests for `.projects` display group (INT-26..INT-50) |
| `cli_cmd_project_path_test.rs` | Spec-driven integration tests for `.project.path` command |
| `cli_cmd_project_exists_test.rs` | Spec-driven integration tests for `.project.exists` command |
| `cli_cmd_session_dir_test.rs` | Spec-driven integration tests for `.session.dir` command |
| `cli_cmd_session_ensure_test.rs` | Spec-driven integration tests for `.session.ensure` command |
| `cli_cmd_tail_test.rs` | Spec-driven integration tests for `.tail` command (INT-1..INT-8) |
| `cli_param_agent_test.rs` | Edge case tests for `agent::` parameter |
| `cli_param_case_sensitive_test.rs` | Edge case tests for `case_sensitive::` parameter |
| `cli_param_count_test.rs` | Edge case tests for `count::` parameter |
| `cli_param_entries_test.rs` | Edge case tests for `entries::` parameter |
| `cli_param_entry_type_test.rs` | Edge case tests for `entry_type::` parameter |
| `cli_param_format_test.rs` | Edge case tests for `format::` parameter |
| `cli_param_limit_test.rs` | Edge case tests for `limit::` parameter |
| `cli_param_metadata_test.rs` | Edge case tests for `metadata::` parameter |
| `cli_param_min_entries_test.rs` | Edge case tests for `min_entries::` parameter |
| `cli_param_output_test.rs` | Edge case tests for `output::` parameter |
| `cli_param_path_test.rs` | Edge case tests for `path::` parameter |
| `cli_param_project_test.rs` | Edge case tests for `project::` parameter |
| `cli_param_query_test.rs` | Edge case tests for `query::` parameter |
| `cli_param_scope_test.rs` | Edge case tests for `scope::` parameter |
| `cli_param_session_id_test.rs` | Edge case tests for `session_id::` parameter |
| `cli_param_sessions_bool_test.rs` | Edge case tests for `sessions::` (bool toggle) parameter |
| `cli_param_session_test.rs` | Edge case tests for `session::` (filter) parameter |
| `cli_param_strategy_test.rs` | Edge case tests for `strategy::` parameter |
| `cli_param_target_test.rs` | Edge case tests for `target::` parameter |
| `cli_param_topic_test.rs` | Edge case tests for `topic::` parameter |
| `cli_param_type_test.rs` | Edge case tests for `type::` parameter |
| `cli_param_show_stat_test.rs` | Edge case tests for `show_stat::` parameter |
| `cli_param_show_tokens_test.rs` | Edge case tests for `show_tokens::` parameter |
| `cli_param_show_tree_test.rs` | Edge case tests for `show_tree::` parameter |
| `cli_param_group_output_control_test.rs` | Cross-command interaction tests for Output Control parameter group |
| `cli_param_group_project_scope_test.rs` | Cross-command interaction tests for Project Scope parameter group |
| `cli_param_group_scope_configuration_test.rs` | Cross-command interaction tests for Scope Configuration parameter group |
| `cli_param_group_session_filter_test.rs` | Cross-command interaction tests for Session Filter parameter group |
| `cli_param_group_session_identification_test.rs` | Cross-command interaction tests for Session Identification parameter group |
| `cli_user_story_audit_session_history_test.rs` | Acceptance tests for Audit Session History user story |
| `cli_user_story_find_past_conversation_test.rs` | Acceptance tests for Find Past Conversation user story |
| `cli_user_story_export_session_for_review_test.rs` | Acceptance tests for Export Session for Review user story |
| `cli_user_story_query_storage_programmatically_test.rs` | Acceptance tests for Query Storage Programmatically user story |
| `cli_user_story_resume_claude_session_test.rs` | Acceptance tests for Resume Claude Session user story |
| `invariant_contracts_test.rs` | Contract tests for IN-/PF-/AL- behavioral invariant, pitfall, and algorithm cases |
| `cli_param_validation_test.rs` | Contract tests for PF-1..4 parameter validation pitfall cases |

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
- Test: `tests/search_command_test.rs::test_search_query_required`
- Fix comment: `src/cli/search.rs` (query parameter validation)

## Integration Test Isolation

All integration tests use `CLAUDE_STORAGE_ROOT` + `TempDir` for full isolation.
No tests depend on real `~/.claude/` state; none are marked `#[ignore]`.

See "Test Isolation with `CLAUDE_STORAGE_ROOT`" below for the pattern.

## Test Naming Conventions

```
test_{command}_{parameter}_{scenario}
```

**Examples**:
- `test_search_query_required` - .search command, query parameter, required validation
- `test_export_format_invalid` - .export command, format parameter, invalid value rejection
- `test_status_path_show_tokens` - .status command, path+show_tokens parameters, interaction

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
- **Specific**: Technical details, not generic statements ("Fixed bug" → "search_routine missing query parameter validation")
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

**Current Status**: 0 ignored
- All tests run fully (none marked `#[ignore]`)
- All tests use `CLAUDE_STORAGE_ROOT` + `TempDir` isolation

## Known Findings

### Finding #009: .count target parameter validation
- **Issue**: Missing validation for target parameter (accepted invalid values)
- **Tests**: 4 tests added in `parameter_validation_test.rs`
- **Fix**: Added validation at `src/cli/mod.rs:1151-1157`
- **Documentation**: Fix(issue-009) comment in source

### Finding #010: .search verbosity parameter validation ✅ Superseded (verbosity removed)
- **Issue**: search_routine missing verbosity range validation (0-5), inconsistent with other commands
- **Test**: Tests removed — verbosity parameter removed from all commands
- **Note**: Verbosity parameter replaced by specific boolean toggles (`show_stat::`, `show_tokens::`, `show_tree::`)

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

### Finding #015: list_routine missing verbosity range validation ✅ Superseded (verbosity removed)
- **Issue**: `list_routine` did not validate verbosity 0-5 range; `-1` or `6` were silently accepted
- **Tests**: Tests removed — verbosity parameter removed from all commands
- **Note**: Verbosity parameter replaced by specific boolean toggles (`show_stat::`, `show_tokens::`, `show_tree::`)

### Finding #016: show_project_routine missing verbosity range validation ✅ Superseded (command removed in task-013, verbosity removed)
- **Issue**: `show_project_routine` did not validate verbosity 0-5 range; same gap as Finding #015
- **Note**: `.show.project` command removed in task-013; verbosity parameter subsequently removed from all commands

### issue-015: .status performance — global_stats() O(total JSONL bytes)
- **Issue**: `.status` took >2 minutes with 1903 projects / 7 GB JSONL
- **Tests**: `status_global_stats_fast_bug.rs` in `claude_storage_core/tests/`
- **Fix**: Added `global_stats_fast()` (filesystem-only); `status_routine` uses it for the default (fast) path
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

### issue-027: list_routine per-project session count uses wrong plural
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

- **Issue**: `.projects` output was a flat list of session IDs with opaque encoded project labels (e.g. `"-home-alice-projects"`); no project grouping, no readable paths, no agent collapse at scale
- **Tests**: 6 tests IT-17..IT-22 in `projects_output_format_test.rs` (IT-23 covers display fix issue-029)
- **Fix**: Redesigned `projects_routine` to group sessions by `BTreeMap<String, Vec<Session>>` keyed by decoded project path; added `decode_project_display()` helper; agent sessions collapsed by default with no `agent::` filter; entry counts shown per session with `show_tree::1`; blank line between project groups
- **Root Cause**: Original design used flat `Vec<(label, id)>` with labels from `format!("{:?}", project.id())` — debug-format encoded strings, not human-readable paths
- **Pitfalls**:
  1. `decode_path()` requires input starting with `-`; UUID project dirs don't → must guard with `starts_with('-')` before calling decode
  2. Topic suffix `--topic` must be stripped (`find("--")`) before calling `decode_path`; otherwise it becomes a phantom path component
  3. Blank line between project groups was not in the initial implementation despite being in the design algorithm and the docs example — always verify format output against docs examples
- **Note**: Originally tagged as issue-026 internally; relabeled to plan-004 because issue-026 was already assigned to the export path-in-error-message bug

### issue-029: decode_project_display splits underscore dirs as path separators

- **Issue**: `.sessions scope::under` displayed project path headers with underscore-named directories split on `/` — e.g., `~/my_project/myproject:` shown as `~/wip/core/myproject:`
- **Test**: `IT-23` (`test_sessions_under_display_preserves_underscores`) in `projects_command_test.rs`; marked `bug_reproducer(issue-029)`
- **Fix**: Added `decode_path_via_fs()` + `walk_fs()` in `cli/mod.rs`; `decode_project_display` now tries the heuristic result first — if it doesn't exist on disk, falls back to FS-guided DFS that chooses `/` vs `_` at each boundary by calling `is_dir()` on candidate prefixes; final fallback is the heuristic result (handles deleted/remote projects)
- **Root Cause**: `encode_path` maps both `_` (underscore) and `/` (path separator) to `-`; `decode_component` heuristic defaulted to `/` for all unrecognized `-` boundaries, so `wip-core` always decoded to `wip/core` regardless of whether a real `my_project` directory exists
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

### issue-034: .projects list mode header count includes zero-byte placeholder sessions

- **Issue**: `clg .projects scope::local` showed `"(2 sessions)"` in the header but rendered 0 session lines when a project had 2 zero-byte placeholder sessions. Same bug in flat display branch (agent:: filter active) and in summary mode (zero-byte file could become the "best session" showing "(no text content)").
- **Tests**: 3 tests IT-54..IT-56 in `projects_zero_byte_count_bug.rs` (use_families branch, flat branch, zero-byte-only project)
- **Fix**: 3-site fix in `src/cli/mod.rs`: (1) `aggregate_projects` skips zero-byte in best-selection and uses `!is_zero_byte_session` in session_count; (2) `root_count` in use_families branch filters to non-zero-byte roots; (3) flat branch computes `displayable` before `group_count`
- **Root Cause**: Count expressions used unfiltered `sessions.len()` / `families.len()` while the render layer had separate `is_zero_byte_session` filtering. Count and render were not derived from the same source
- **Documentation**: Fix(issue-034) 3-field comment at all three source sites + 5-section test docs in `projects_zero_byte_count_bug.rs`

### issue-037: `.session.dir` and `.session.ensure` rejected absent `path::` despite YAML spec declaring it optional

- **Issue**: `clg .session.dir` without `path::` exited 1 with an error even though the YAML spec marks `path::` as `optional: true` with "default: current directory"
- **Tests**: `it_session_dir_cwd_default`, `it_session_ensure_cwd_default` in `session_path_command_test.rs`; both marked `bug_reproducer(issue-037)`
- **Fix**: Replaced `ok_or_else` + `resolve_path_parameter` call chain in `resolve_session_dir` with a single call to `resolve_cmd_path(cmd)?`, which returns `cwd` when `path::` is absent
- **Root Cause**: `resolve_required_session_dir` (old name) used `cmd.get_string("path").ok_or_else(|| error)?` which unconditionally rejected absent `path::` before any cwd fallback could occur. The sibling helper `resolve_cmd_path` already implemented the correct pattern
- **Documentation**: Fix(issue-037) in `cli/mod.rs` `resolve_session_dir`; 5-section test doc in `session_path_command_test.rs`

### issue-036: `.show session_id::` fails for sessions in topic project directories

- **Issue**: `clg .show session_id::UUID` returned "Session not found" for sessions recorded in topic dirs (e.g., `-commit`, `-default_topic`) even though `.projects` showed the session under the current project
- **Tests**: `show_finds_session_in_topic_dir`, `show_rejects_sibling_single_hyphen_dir` and others in `content_display_integration_test.rs`; marked `bug_reproducer(issue-036)`
- **Fix**: Replaced `storage.load_project_for_cwd()` in `show_session_in_cwd_impl` with a `list_projects()` scan filtered by `dir_name == eb || dir_name.starts_with(&format!("{eb}--"))` (double-hyphen prevents sibling matches)
- **Root Cause**: `load_project_for_cwd()` matched only the exact encoded base path; topic dirs use `{base}--{topic}` suffixes, which exact-match never returned
- **Documentation**: Fix(issue-036) in `cli/mod.rs` `show_session_in_cwd_impl`; 5-section test doc in `content_display_integration_test.rs`

### issue-035: `.projects` shows base path instead of topic path when topic directory is absent from disk

- **Issue**: `clg .projects` displayed `project/base` instead of `project/base/-commit` for sessions recorded under a topic dir that no longer existed on disk
- **Tests**: `projects_shows_topic_path_when_topic_dir_absent`, `projects_shows_topic_path_when_topic_dir_exists` and others in `projects_path_encoding_test.rs`; marked `bug_reproducer(issue-035)`
- **Fix**: Removed the `if candidate.exists()` guard inside the topic-extension loop in `decode_project_display` — always join the topic component unconditionally
- **Root Cause**: The existence check treated disk state as authoritative for display; but the storage key records the CWD at session start, and that attribution must not change based on current filesystem state
- **Documentation**: Fix(issue-035) in `cli/mod.rs` `decode_project_display`; 5-section test doc in `projects_path_encoding_test.rs`

### issue-028: "1 entries" — hardcoded plural "entries" in session header and project session list
- **Issue**: (a) `.show session_id::abc` produced "Session: abc (1 entries)" — wrong plural in header; (b) `.show.project` with 1-entry session showed "(1 entries, last: ...)" — same root cause
- **Tests**: `test_show_session_single_entry_header_says_entry_not_entries`, `test_show_session_multi_entry_header_still_says_entries` in `smart_show_command.rs`; `.show.project` tests deleted with command removal (task-013)
- **Fix**: Added `entry_noun`/`e_noun` variables derived from count (1 → "entry", else "entries") in `show_session_routine` in `cli/mod.rs`; same fix was in `show_project_routine` (now removed)
- **Root Cause**: Format strings hardcoded "entries" regardless of count — same pattern as issue-025/027 but for the irregular noun "entry"/"entries"
- **Documentation**: Fix(issue-028) in `cli/mod.rs` (two locations) + 5-section test doc in both test files

## Manual Testing

See `tests/manual/readme.md` for manual testing plan and procedures.

## Related Documentation

- **Code Design**: See applicable rulebooks via `clm .rulebooks.list`
- **Test Organization**: `test_organization.rulebook.md` - Test documentation format standards
- **Codebase Hygiene**: `codebase_hygiene.rulebook.md` - Quality standards for documentation
