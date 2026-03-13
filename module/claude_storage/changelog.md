# Changelog

All notable changes to claude_storage will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`.sessions` command — scope-aware session listing** (2026-03-28)
  - New command providing a session-first view (flat list, not grouped by project)
  - `scope::local` (default) — sessions for the exact matching project only
  - `scope::relevant` — sessions from all ancestor projects of `path::` (mirrors `kbase`)
  - `scope::under` — sessions from all projects under `path::` (subtree)
  - `scope::global` — all sessions across entire storage (UUID projects included)
  - Filters: `session::`, `agent::`, `min_entries::` (same semantics as `.list`)
  - Verbosity 0: raw IDs; verbosity 1: `Found N sessions:` header; verbosity 2+: IDs with project labels
  - 16 tests covering EC-1..EC-8 edge cases + behavioural and validation tests

- **`clg` alias binary** (2026-03-28)
  - Added `[[bin]] name = "clg"` in `Cargo.toml` pointing to `src/main.rs`
  - `clg` and `claude_storage` are identical binaries; `clg` is a short-name alias for interactive shell use
  - `tests/common/mod.rs`: added `clg_cmd()` helper + Binary Name Coupling doc comment
  - `tests/cli_sanity.rs`: added `clg_alias_is_present` and `clg_alias_matches_claude_storage` smoke tests

- **REQ-012: Search Command Specification** (2025-12-06)
  - Full-text search across conversation content
  - Parameters: query (required), project, session, case_sensitive, entry_type, verbosity
  - Validation requirements documented (V-012.1 through V-012.7)
  - Command implemented and functional

- **Phase 1B: .search parameter validation tests** (2025-12-06)
  - Added 8 comprehensive tests in tests/search_command_test.rs
  - Tests cover: query required, query empty, case_sensitive, entry_type, verbosity validation
  - 5 tests passing, 3 tests ignored (integration tests for future use)

- **REQ-013: Export Command Specification** (2025-12-06)
  - Export session data to Markdown, JSON, or Plain Text formats
  - Parameters: session_id (required), format, output (required), project
  - Validation requirements documented (V-013.1 through V-013.5)
  - Command defined in YAML but not yet implemented

- **Phase 1C: .export parameter validation tests** (2025-12-06)
  - Added 8 comprehensive tests in tests/export_command_test.rs
  - Tests cover: session_id required, output required, format validation
  - 3 tests passing, 5 tests ignored (integration tests for future use)

- **Phase 1D: .status path parameter tests** (2025-12-06)
  - Added 5 comprehensive tests in tests/status_path_test.rs
  - Tests cover: default path, custom path, nonexistent path, empty path, path with verbosity
  - 4 tests passing, 1 test ignored (integration test for default path behavior)

- **.status path parameter documentation** (2025-12-06)
  - Documented existing `path::` parameter for custom storage paths
  - Examples added to spec.md

- **CLAUDE_STORAGE_ROOT env var for storage path override** (hygiene sprint)
  - `CLAUDE_STORAGE_ROOT` env var overrides default `~/.claude/` storage path
  - Precedence: `path::` param > `CLAUDE_STORAGE_ROOT` > `~/.claude/`; empty string falls back to default
  - Implemented via `create_storage()` helper in `src/cli/mod.rs`; replaces 9 `Storage::new()` call sites
  - Primary use case: test isolation without touching real storage

- **`write_test_session()` and `write_path_project_session()` test helpers** (hygiene sprint)
  - Added to `tests/common/mod.rs` for fixture creation in isolated `TempDir` environments
  - Enables all 18 previously-ignored integration tests to run without `~/.claude/` dependency

- **`tests/list_command_test.rs` — `.list` parameter coverage** (hygiene sprint)
  - New test file covering `.list` parameter bounds and combinations (22 tests)
  - Covers: `agent::`, `session::`, `type::`, `verbosity::` bounds (Finding #015), `path::`, `sessions::`, `min_entries::`, pairwise combinations

### Fixed

- **Finding #015: list_routine missing verbosity range validation** (hygiene sprint)
  - `list_routine` accepted verbosity values outside 0-5 range without error
  - Fix: added range check matching `status_routine` pattern
  - Root cause: verbosity extracted but bounds check never added, unlike other routines
  - Tests in `list_command_test.rs`

- **Finding #016: show_project_routine missing verbosity range validation** (hygiene sprint)
  - `show_project_routine` accepted verbosity values outside 0-5 range without error
  - Fix: added range check matching `status_routine` pattern
  - Root cause: same gap as Finding #015 in a separate routine
  - Tests in `show_project_command.rs` (N: verbosity::-1, verbosity::6; P: verbosity::0, verbosity::5)

- **Finding #009: .count target parameter validation**
  - Added 4 comprehensive tests for .count target validation
  - Tests cover: invalid values, valid values (projects/sessions/entries), singular form typo, empty value
  - Added Fix(issue-009) comment documenting root cause and pitfall
  - All tests passing

- **Finding #010: .search verbosity parameter validation** (2025-12-06)
  - search_routine was missing verbosity range validation (0-5), inconsistent with status_routine and show_routine
  - Added validation at src/cli/mod.rs:1190 matching pattern used in other commands
  - Root cause: Assumed default values prevent invalid input (they don't)
  - Added Fix(issue-010) comment documenting root cause and pitfall
  - Test now passing (test_search_verbosity_invalid)

- **Finding #011: Partial UUID matching for .show session_id parameter** (2025-12-06)
  - Session lookup in .show and .export commands only supported exact UUID matching, not prefix matching
  - Users had to type full 36-character UUID instead of convenient 8-character prefix (e.g., "79f86582" vs "79f86582-1435-442c-935a-13f8d874918a")
  - Root cause: Implementation only checked `s.id() == session_id` without checking `s.id().starts_with(session_id)`
  - Fix applied to both format_session_output (line 892) and export routine (line 1448) with Fix(issue-011) comments
  - Added comprehensive test test_show_partial_uuid_matching with 5-section bug documentation
  - Pitfall: Test data that doesn't match production patterns (using "test-session-123" instead of real UUIDs) leads to missing coverage
  - All tests passing (91 tests: 82 passing, 9 ignored)

- **Finding #012: ProjectId parsing bug in .count/.search/.export commands** (2025-12-06)
  - Commands .count, .search, and .export hardcoded `ProjectId::uuid(proj_id)` preventing path projects from working
  - Users with path projects got "Project not found" errors because commands looked for UUID-named directories instead of path-encoded directories
  - Root cause: Same bug as Finding #008 (.show command fix) but not propagated to other commands
  - Affected locations: count_routine (lines 1171, 1187), search_routine (lines 1280, 1307), export_routine (line 1436)
  - Fix: Replaced all `ProjectId::uuid(proj_id)` with `parse_project_parameter(proj_id)?` at 5 locations
  - Added Fix(issue-012) comments at each location documenting root cause and pitfall
  - Added comprehensive test file tests/project_parameter_multi_command_bug.rs with 5-section documentation
  - Created 2 active tests (test_count_with_path_project, test_count_entries_with_path_project) and 3 ignored tests for future use
  - Pitfall: When fixing a bug in one command, grep for identical patterns across the entire codebase - bugs often exist in multiple locations sharing the same flawed assumption
  - All tests passing (95 tests: 84 passing, 11 ignored)

- **issue-030: `.sessions` path header truncated at topic directory** (2026-03-31)
  - Session path headers showed only the base directory, truncating hyphen-prefixed topic components even when they represent real directories (e.g., `src/-default_topic` displayed as `src`)
  - Root cause: `decode_project_display` stripped all `--topic` suffixes before decoding, discarding the topic component entirely
  - Fix: `decode_project_display` now tries to extend the decoded base path with each topic component as a real filesystem directory; the longest existing path is used as the display header
  - Regression test: `it_24_decode_display_includes_hyphen_prefixed_topic_dir` (`sessions_command_test.rs`)

- **issue-031: `scope::under` false positive for sibling modules with underscore names** (2026-03-31, TSK-060)
  - `scope::under` with base `claude_storage` incorrectly included sessions from sibling module `claude_storage_core`
  - Root cause: `encode_path` maps both `_` and `/` to `-`; string `starts_with` on encoded forms cannot distinguish child `base/sub` (encodes `base-sub`) from sibling `base_extra` (encodes `base-extra`), as both share the `base-` prefix
  - Fix: two-stage predicate — string prefix is fast-reject only; candidates that pass string check are verified via `decode_path_via_fs` + `Path::starts_with` (component-wise), which correctly excludes siblings
  - Regression test: `it_25_scope_under_excludes_underscore_named_sibling` (`sessions_command_test.rs`)

- **issue-032: `scope::relevant` false positive for sibling projects with underscore-suffix names** (2026-03-31)
  - `scope::relevant path::base_extra` incorrectly included sessions from sibling project `base` — `/base` is not an ancestor of `/base_extra`
  - Root cause: `is_relevant_encoded` used `encoded_base.starts_with(dir_name + "-")` — symmetric form of issue-031's bug: a sibling project whose encoded name is a string prefix of the current path's encoded form was falsely treated as an ancestor
  - Fix: same two-stage predicate as issue-031 applied to the `"relevant"` arm — `is_relevant_encoded` as fast-reject; `decode_path_via_fs` + `base_path.starts_with(decoded_path)` (component-wise) for prefix-match candidates
  - Regression test: `it_26_scope_relevant_excludes_underscore_named_sibling` (`sessions_command_test.rs`)

### Changed

- **Test isolation strategy**: Moved from `#[ignore]` + real `~/.claude/` to `CLAUDE_STORAGE_ROOT` + `TempDir` (hygiene sprint)
  - Integration tests no longer require real Claude Code storage state
  - `#[ignore]` eliminated; all tests pass in any environment including CI
  - Subprocess env var (`cmd.env(…)`) used instead of `std::env::set_var()` to avoid parallel-test races

- **Command count**: Updated from 5 to 7 commands (added .search and .export)
- **Command routines**: Added search_routine and export_routine to architecture documentation
- **Known limitations**: Removed "No search" and "Output format" (now addressed by REQ-012 and REQ-013)
- **Future enhancements**: Updated priority list to reflect completed search and export specifications

### Test Coverage

- Test count increased from 65 to 149 tests (149 passing, 0 ignored) — hygiene sprint (2026-03-26)
- Test count increased from 149 to 155 tests (155 passing, 0 ignored) — L5 CLI documentation verification (2026-03-27)
  - `count_command_bug_fix.rs` +1 test (issue-003a/b, issue-017)
  - `parameter_validation_test.rs` +1 test (cross-command param validation)
  - `search_command_test.rs` +1 test (REQ-012 edge case)
  - `search_session_partial_uuid_bug.rs` +2 tests (new file — partial UUID match bug reproducer)
  - `status_path_test.rs` +1 test (path parameter edge case)
- Parameter coverage: 100% of all 30 CLI parameters have validation tests (hygiene sprint)
- Added 4 parameter validation tests for .count target parameter (Finding #009)
- Added 8 parameter validation tests for .search command (Phase 1B)
- Added 8 parameter validation tests for .export command (Phase 1C)
- Added 5 parameter validation tests for .status path parameter (Phase 1D)
- Added 1 comprehensive test for partial UUID matching (Finding #011)
- Added 4 comprehensive tests for project parameter bug (Finding #012)
- Added 22 tests for .list parameter bounds and combinations (Finding #015, hygiene sprint)
- Added 4 verbosity tests for .show.project (Finding #016, hygiene sprint)
- Added 2 tests for .count session filter (hygiene sprint)
- Added 2 tests for .session path parameter (hygiene sprint)
- Eliminated all 18 `#[ignore]` tests — rewrote with CLAUDE_STORAGE_ROOT + TempDir isolation
- Fixed 6 validation bugs (Finding #009: target, #010: search verbosity, #011: partial UUID, #012: project param, #015: list verbosity, #016: show_project verbosity)

- Test count increased to 212 tests (212 passing, 0 ignored) — issue-030/031 regression tests (2026-03-31, TSK-060)
  - `sessions_command_test.rs`: +2 tests (`it_24_decode_display_includes_hyphen_prefixed_topic_dir`, `it_25_scope_under_excludes_underscore_named_sibling`)
  - Total `sessions_command_test.rs`: 43 tests

- Test count increased to 213 tests (213 passing, 0 ignored) — issue-032 regression test (2026-03-31)
  - `sessions_command_test.rs`: +1 test (`it_26_scope_relevant_excludes_underscore_named_sibling`)
  - `sessions_output_format_test.rs`: new file (6 tests extracted from `sessions_command_test.rs` to keep file under 1500-line limit)
  - Net: +1 test (split moves 6 existing tests to new file, adds 1 new test)

### Phase 1 Verification (2025-12-06)

- ✅ All Phase 1 deliverables verified (test count, file existence, coverage calculations)
- ✅ Coverage calculation corrected from ~58% to 76% (22/29 parameters)
- ✅ All 95 tests verified: 84 passing, 11 properly ignored
- ✅ No missing deliverables or pending work
- ✅ Plan updated with accurate numbers and verification report
- ✅ Manual testing completed (Finding #011 and #012 discovered and fixed with TDD)
- ✅ Full TDD cycle completed: RED → GREEN → VERIFY for all findings
- ✅ All tests pass with `w3 .test l::3` (100% success rate, zero warnings)

## [1.3.0] - 2025-12-05

### Changed (BREAKING)

- **Content-first display (REQ-011)**: `.show` command now displays conversation content by default when showing a specific session
  - **Breaking change**: Default behavior changed from metadata-only to full conversation content
  - **Root cause**: Users primarily want to read conversations, not inspect metadata
  - **Migration**: Use `metadata::1` parameter to get old metadata-only behavior
  - **Impact**: Much better UX - content is now immediately visible without needing `entries::1`
  - **Backward compatibility**: `metadata::1` parameter preserves old behavior for users who need it

### Added

- **Content formatting functions**:
  - `format_entry_content()` - Extracts and formats conversation messages from entries
  - `format_timestamp()` - Converts ISO 8601 timestamps to human-readable format (YYYY-MM-DD HH:MM)
  - `truncate_if_needed()` - Smart text truncation with ellipsis for long content
- **Verbosity levels redesigned**:
  - `verbosity::0` - Metadata only (equivalent to `metadata::1`)
  - `verbosity::1` - **Full conversation content** (NEW default)
  - `verbosity::2` - Conversation + metadata header
  - `verbosity::3+` - Conversation + metadata + extended details
- **Chat-log format**: Clean, readable conversation display with role labels and timestamps
- **metadata::1 parameter**: Explicit parameter to show metadata-only (old default behavior)

### Changed

- **Command versions**: Updated `.show` and `.show.project` to v1.3.0 for consistency with crate release

### Fixed

- **Parameter validation**: Fixed 4 parameter validation gaps discovered during manual testing:
  - `.list type::invalid` - Now rejects invalid type values with clear error message (valid: uuid, path, all)
  - `.status verbosity::-1` - Now rejects negative verbosity values (valid range: 0-5)
  - `.status verbosity::10` - Now rejects out-of-range verbosity values (valid range: 0-5)
  - `.list min_entries::-5` - Now rejects negative min_entries values (must be non-negative)
  - **Root cause**: Missing application-level validation for parameter values (relied only on type checking)
  - **Impact**: Silent failures, user confusion, incorrect behavior with invalid parameter values
  - **Solution**: Explicit validation with clear error messages for all parameter value ranges

### Test Coverage

- Added 3 comprehensive integration tests for content display behavior:
  - `show_displays_content_by_default()` - Verifies content shown by default
  - `show_metadata_only_parameter()` - Verifies metadata::1 works
  - `show_verbosity_zero_is_metadata_only()` - Verifies verbosity::0 equivalence
- Added 2 comprehensive tests for command version consistency
- Added 8 comprehensive tests for parameter validation:
  - `.list type::` parameter validation (accepts uuid, path, all; rejects invalid)
  - `.status verbosity::` range validation (0-5, rejects negative and out-of-range)
  - `.show verbosity::` range validation (0-5, rejects negative and out-of-range)
  - `.list min_entries::` validation (rejects negative values)
- All tests passing (63/63 total, up from 50)
- 7-stage validation framework created with 104 automated checks
- Manual testing discovered and fixed 4 parameter validation gaps

### Migration Guide

- **Default usage** (no changes needed):
  ```bash
  # Before: .show session_id::X entries::1    # Had to specify entries::1
  # After:  .show session_id::X                # Content shown by default
  ```
- **If you want metadata only**:
  ```bash
  .show session_id::X metadata::1             # Explicit metadata-only mode
  ```
- **Backward compatibility**: `metadata::1` parameter provides exact old behavior

## [1.2.1] - 2025-12-01

### Fixed

- **`.list` session filter bug**: Fixed garbage parameter issue where `session::`, `agent::`, and `min_entries::` parameters were accepted but silently ignored
  - **Root cause**: `show_sessions` defaulted to false, blocking filter usage even when filters provided
  - **Impact**: Users wasted time trying different parameter values with no effect
  - **Solution**: Smart auto-enable - session display automatically enabled when session filters provided
  - **Behavior**: When any session filter is provided (`session::`, `agent::`, `min_entries::`), sessions are automatically shown
- **`.list` robustness**: Fixed race condition where listing would fail if a project was deleted while iterating
  - **Root cause**: Hard errors when counting sessions for deleted projects
  - **Impact**: Parallel test execution would fail intermittently
  - **Solution**: Gracefully skip projects that can't be read instead of failing entire operation
  - **Behavior**: Deleted or inaccessible projects are silently skipped during listing

### Changed

- **Smart session display**: `.list` now auto-enables session display when session filters provided
- **Progressive disclosure**: No need to specify `sessions::1` when using session filters (auto-detected)

### Test Coverage

- Added 6 comprehensive tests for smart session display behavior
- All CLI tests passing (29 previous + 6 new = 35 total)
- Test coverage includes: no filters, auto-enable cases, explicit enable, multiple filters

### Migration

- **No breaking changes**: Existing `sessions::1` usage still works
- **Recommended**: Remove redundant `sessions::1` when using session filters (auto-enabled)
- **Examples**:
  - Old: `.list sessions::1 session::commit` → New: `.list session::commit` (shorter, same result)
  - Old: `.list sessions::1 agent::1` → New: `.list agent::1` (shorter, same result)

## [1.2.0] - 2025-11-30

### Added

- **Location-aware `.show` command**: Smart parameter detection adapts behavior based on what parameters are provided:
  - No parameters → Shows current directory project (all sessions)
  - `session_id` only → Shows that session in current project
  - `project` only → Shows that project (all sessions)
  - Both parameters → Shows that session in that project (original behavior)
- **Progressive disclosure UX**: Common case (current directory) requires no parameters, detailed cases available when needed
- **Design principle documented**: Added "location-aware commands" section to spec.md

### Changed

- **`.show.project` deprecated**: Soft deprecation in favor of `.show` (same functionality, better UX)
- **Optional parameters**: Both `session_id` and `project` parameters are now optional in `.show` command
- **YAML version**: Updated `.show` and `.show.project` to v1.2.0

### Test Coverage

- Added 6 comprehensive tests for smart `.show` behavior
- All project-related tests passing (show_project_command, smart_show_command, project_parameter_bug_fix)

### Migration

- Replace `.show.project` usage with `.show` or `.show project::{path}`
- `.show.project` still works but shows deprecation notice

## [1.1.0] - 2025-11-30

### Added

- **`.show.project` command**: Display project details and all sessions without needing specific session UUID
  - Accepts multiple project identifier formats (path, path-encoded, UUID, Path(...) from .list output)
  - Shows project statistics and lists all sessions with their entry counts
- **Smart project parameter parsing**: Enhanced `.show` command to accept various project identifier formats
  - Copy-paste friendly: Can paste `Path("/...")` directly from `.list` output
  - Flexible format detection (absolute path, path-encoded, UUID)

### Changed

- Enhanced project parameter handling in `.show` command
- Improved error messages for invalid project identifiers

### Test Coverage

- Added 3 tests for `.show.project` command
- Added 3 tests for project parameter parsing
- All integration tests passing

## [1.0.2] - 2025-11-29

### Fixed

- Build system improvements
- Documentation updates

## [1.0.1] - 2025-11-28

### Fixed

- Minor bug fixes
- Performance improvements

## [1.0.0] - 2025-11-27

### Added

- Initial release of claude_storage CLI
- `.status` command - Show storage statistics
- `.list` command - List projects and sessions
- `.show` command - Display session details
- `.count` command - Fast counting operations
- `.search` command - Search session content
- `.export` command - Export sessions to file (markdown, JSON, text)
- Project and session filtering capabilities
- Path-encoded project identifiers for filesystem-safe storage
