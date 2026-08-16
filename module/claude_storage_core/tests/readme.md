# Test Suite Organization

## Overview

The claude_storage_core test suite covers the core storage library: JSON parsing, path
encoding/decoding, session filtering, content search, and export. The suite is split between
integration tests that exercise real `~/.claude/` storage and unit tests that run
fully in-process. Ten of the seventeen files are bug reproducers — each documents a parse,
encoding, or storage defect found in production data with 5-section root-cause documentation.
`status_global_stats_fast_bug.rs` covers both issue-015 (performance) and issue-018 (agent
session discovery for Claude Code v2.x format) with corner case tests for subagents/ traversal.

## Test Structure

```
tests/
├── readme.md                              # This file — test suite organization
├── scope_test.rs                          # Unit tests for scope_for(), git_root_for(), ClaudeScope
├── continuation_tests.rs                  # Integration tests for continuation detection and UUID selection
├── session_id_tests.rs                    # Unit tests for SessionId newtype
├── count_entries_bug.rs                   # Bug Reproducer (issue-016): count_entries vs stats mismatch
├── export.rs                              # Export integration tests (markdown, JSON, text)
├── filtering.rs                           # Session and project filtering integration tests
├── is_agent_session_doc_mismatch_bug.rs   # Bug Reproducer (BUG-491): doc comment claimed isSidechain check that never existed
├── json_multibyte_bug.rs                  # Bug Reproducer (bug-1): byte/char index mismatch
├── json_surrogate_pair_bug.rs             # Bug Reproducer (issue-001): UTF-16 surrogate pairs
├── path_decoding_hyphen_component_bug.rs  # Bug reproducer: hyphen-prefixed component decoding
├── path_encoding_double_slash_bug.rs      # Bug reproducer: double-slash from lossy encoding
├── search.rs                              # Content search integration tests
├── sessions_filtered_corrupted_session_bug.rs # Bug Reproducer (BUG-492): sessions_filtered() discarded project on one corrupted session
├── stats_malformed_line_bug.rs            # Bug Reproducer (BUG-489): stats() hard-fail on malformed line
├── status_global_stats_fast_bug.rs        # Bug Reproducer (issue-015): global_stats() performance
├── string_matcher.rs                      # StringMatcher unit tests (case-insensitive matching)
└── underscore_encoding_compatibility.rs   # Bug reproducer: underscore vs hyphen encoding mismatch
```

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `scope_test.rs` | Unit tests for `scope_for()`, `git_root_for()`, and `ClaudeScope` path computation |
| `continuation_tests.rs` | Integration tests for `check_continuation`, `most_recent_session_id`, `most_recent_session_in_dir`, and `to_storage_path_for` |
| `session_id_tests.rs` | Unit tests for `SessionId` newtype: construction, display, clone, and `From` conversions |
| `count_entries_bug.rs` | Reproduce and verify fix for count_entries() vs stats() mismatch |
| `export.rs` | Integration tests for session export (markdown, JSON, text formats) |
| `filtering.rs` | Session and project filter composition integration tests |
| `is_agent_session_doc_mismatch_bug.rs` | Lock in filename-only `is_agent_session()` contract; regression guard for BUG-491 |
| `json_multibyte_bug.rs` | Reproduce and verify fix for multi-byte UTF-8 parser bug |
| `json_surrogate_pair_bug.rs` | Reproduce and verify fix for UTF-16 surrogate pair parsing |
| `path_decoding_hyphen_component_bug.rs` | Reproduce and verify fix for hyphen component decode |
| `path_encoding_double_slash_bug.rs` | Reproduce and verify fix for lossy path encoding |
| `search.rs` | Content search across sessions integration tests |
| `sessions_filtered_corrupted_session_bug.rs` | Lock in that one corrupted session must not discard a project's other valid sessions; regression guard for BUG-492 |
| `stats_malformed_line_bug.rs` | Reproduce and verify fix for stats() hard-fail on malformed JSONL line |
| `status_global_stats_fast_bug.rs` | Reproduce and verify fix for global_stats() performance bug |
| `string_matcher.rs` | Unit tests for StringMatcher case-insensitive substring matching |
| `underscore_encoding_compatibility.rs` | Reproduce and verify fix for underscore/hyphen encoding |

## Test Documentation Standards

### Feature Tests (New Functionality)

Use 4-section Purpose format:

```rust
/// Test {functionality} {scenario}
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
/// {Spec section or REQ-NNN this test validates}
#[test]
fn test_{functionality}_{scenario}()
```

**Examples**:
- `tests/search.rs::search_basic_case_insensitive`
- `tests/filtering.rs::session_filter_agent_only`
- `tests/export.rs::export_markdown_basic`

### Bug Fix Tests (Bug Reproducers)

Use 5-section Root Cause format:

```rust
/// Test {component} {issue} (Bug Reproducer: issue-NNN / bug-N)
///
/// ## Root Cause
/// {Technical explanation of why bug occurred}
///
/// ## Why Not Caught
/// {Gap in existing tests that allowed bug}
///
/// ## Fix Applied
/// {What code change resolved the issue}
///
/// ## Prevention
/// {Policy to prevent similar bugs}
///
/// ## Pitfall
/// {Anti-pattern that caused bug}
#[test]
fn test_{component}_{issue}()
```

**Source Code Fix Comment** (3 required fields):
```rust
// Fix(issue-NNN): {One-line description}
//
// Root cause: {Why bug occurred}
//
// Pitfall: {Anti-pattern to avoid}
```

**Examples**:
- Test: `tests/json_surrogate_pair_bug.rs` — issue-001 documentation
- Test: `tests/json_multibyte_bug.rs` — bug-1 documentation
- Fix comment: `src/json.rs` (byte/char index fix for bug-1)

## Integration Test Strategy

Tests that depend on real `~/.claude/` storage state must skip gracefully at runtime
(not via `#[ignore]`) when the storage is absent or empty:

```rust
#[test]
fn test_storage_real_data()
{
  let storage = Storage::new().expect( "open storage" );
  let projects = storage.list_projects().unwrap_or_default();
  if projects.is_empty()
  {
    println!( "SKIP: no projects in real storage" );
    return;
  }
  // ... actual assertions ...
}
```

**Why**:
- `#[ignore]` disables tests permanently — graceful skip runs the test in all environments
- Machines with real storage get meaningful coverage; machines without get a transparent skip
- Avoids "garbage parameter" anti-pattern (test accepted but never runs)

**Examples**:
- `tests/export.rs::export_markdown_basic` — skips gracefully when no projects found
- `tests/filtering.rs::session_filter_agent_only` — skips gracefully when no sessions found

## Test Naming Conventions

```
{component}_{scenario}
```

**Examples**:
- `search_basic_case_insensitive` — search component, basic case-insensitive scenario
- `session_filter_agent_only` — session filter, agent_only parameter scenario
- `export_markdown_basic` — export component, markdown format basic scenario
- `json_parser_multibyte_utf8` — JSON parser, multi-byte UTF-8 scenario

## Test Organization Principles

### Domain-Grouped Files

Tests are grouped by functional domain, not by test type:

- `export.rs` — all export-related tests
- `filtering.rs` — all filter-related tests (session + project)
- `search.rs` — all search-related tests

### Bug Reproducers as First-Class Tests

Bug reproducers are permanent fixtures, not temporary debugging files:
- One file per bug/issue, named after the specific defect
- 5-section documentation required (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
- Tests remain to prevent regression after fix is applied

### Unit Tests for Utility Types

Pure algorithmic utilities without storage I/O get dedicated unit test files:
- `string_matcher.rs` — zero-dependency, runs fully in-process

## Test Quality Standards

### Documentation Quality

Test documentation must be:
- **Specific**: Technical details, not generic ("byte/char index mismatch in `json.rs:peek()`" not "parser bug")
- **Actionable**: Clear prevention steps ("always use byte-oriented indexing for UTF-8")
- **Traceable**: Links to issue IDs (bug-1, issue-001), source locations (`src/json.rs:288-289`)
- **Concise**: Essential information only, no redundancy

### No Silent Failures

Integration tests that depend on real storage must skip gracefully:
- Print a clear message when data is absent (`println!("SKIP: ...")`)
- Return immediately without failing the test
- Never use `#[ignore]` — that disables the test permanently

Never silently pass — that masks missing test coverage.

### No Mocking

Tests must use real implementations:
- ✅ `Storage::new()` for real `~/.claude/` storage
- ✅ Graceful skip at runtime when data is absent (not `#[ignore]`)
- ❌ Don't mock Storage, Session, or JSON parsing

## Test Verification Commands

```bash
# Run all effective tests (excludes ignored)
RUSTFLAGS="-D warnings" cargo nextest run --all-features

# Run single test file
cargo nextest run --test json_surrogate_pair_bug --all-features

# Run ignored tests only
cargo nextest run --all-features -- --ignored

# Run all tests including ignored
cargo nextest run --all-features -- --include-ignored
```

## Known Bug Reproducers

### bug-1: JSON Multi-Byte UTF-8 Byte/Char Index Mismatch
- **File**: `json_multibyte_bug.rs`
- **Component**: `src/json.rs` — custom JSON parser
- **Issue**: `self.position` used as both byte index and char index; diverged for multi-byte chars (em-dash = 3 bytes)
- **Fix**: Changed `peek()` to use byte slicing + `chars().next()`; `advance()` uses `char::len_utf8()`
- **Root Cause**: All prior parser tests used ASCII-only JSON; real Claude Code data contains Unicode

### issue-001: UTF-16 Surrogate Pair Handling
- **File**: `json_surrogate_pair_bug.rs`
- **Component**: `src/json.rs:288-289` — `\uXXXX` escape sequence parsing
- **Issue**: `char::from_u32()` fails for high surrogates (U+D800–U+DBFF); needed surrogate-pair combination
- **Fix**: Detect high surrogate, read low surrogate, combine: `0x10000 + ((high & 0x3FF) << 10) + (low & 0x3FF)`
- **Root Cause**: Parser assumed each `\uXXXX` was a standalone code point; emojis use surrogate pairs

### Hyphen Component Decoding Bug
- **File**: `path_decoding_hyphen_component_bug.rs`
- **Component**: `src/path.rs::decode_component()`
- **Issue**: `--default-topic` decoded as TWO components (`-default`, `topic`) instead of ONE (`-default-topic`)
- **Fix**: Enhanced heuristic decoder to use context (after `module/`) for component boundary detection
- **Root Cause**: Simple `--` detection didn't account for internal hyphens in hyphen-prefixed names

### Path Encoding Double-Slash Bug
- **File**: `path_encoding_double_slash_bug.rs`
- **Component**: `src/path.rs` — path encoding/decoding
- **Issue**: Lossy encoding (`/` and `_` both → `-`) caused old decoder (replace all `-` with `/`) to produce double slashes; affected 89% of projects with hyphen-prefixed directories
- **Fix**: Recognize `--` as `/-` prefix to restore hyphen-prefixed directory components

### issue-016: count_entries() Counted All JSONL Lines, Not Conversation Entries
- **File**: `count_entries_bug.rs`
- **Component**: `src/session.rs::count_entries()`
- **Issue**: `.count target::entries` returned 2135 while `.show` "Total Entries" showed 2034 for the same session — a discrepancy of 101 metadata lines
- **Fix**: Changed `count_entries()` to parse `"type"` field and count only `"user"`/`"assistant"` entries, matching `stats().total_entries`
- **Root Cause**: Original implementation used `content.lines().count()` — counted every non-empty JSONL line including internal metadata (queue-operation, system, summary)

### issue-015: global_stats() Performance — JSONL Parsing O(total_bytes)
- **File**: `status_global_stats_fast_bug.rs`
- **Component**: `src/storage.rs::global_stats()` + `cli/mod.rs::status_routine()`
- **Issue**: `.status` took >2 minutes with 1903 projects / 7 GB JSONL because `global_stats()` parsed every session file to count entries and tokens
- **Fix**: Added `global_stats_fast()` (filesystem metadata only; no JSONL parsing); `status_routine` uses it for verbosity 0-1. `global_stats()` only called at verbosity 2+ when full stats explicitly requested.
- **Root Cause**: `project_stats()` called `session.stats()` for every session, which reads + parses JSONL. Complexity is O(total_JSONL_bytes), not O(project_count)

### issue-018: Agent Sessions in New Claude Code v2.x Format Were Invisible
- **File**: `status_global_stats_fast_bug.rs`
- **Component**: `src/project.rs::iter_session_files()`
- **Issue**: `global_stats_fast()` reported `Agent: 0` even with 11,757 agent session files; `all_sessions()` missed all new-format agent sessions
- **Fix**: Extended `iter_session_files()` to traverse `{project_dir}/{uuid}/subagents/agent-*.jsonl` when `include_agents=true`; keeps backward compat with old format (`{project_dir}/agent-*.jsonl`)
- **Root Cause**: Claude Code v2.x changed agent session storage from `{project_dir}/agent-{id}.jsonl` to `{project_dir}/{uuid}/subagents/agent-{id}.jsonl`; iterator only scanned the top-level project directory
- **Corner Cases**: `sessions_main_only_excludes_new_format_agents` (include_agents=false guard), `global_stats_fast_ignores_non_jsonl_in_subagents` (noise tolerance), `global_stats_fast_empty_subagents_dir` (empty dir)

### Underscore Encoding Compatibility
- **File**: `underscore_encoding_compatibility.rs`
- **Component**: `src/path.rs` — path encoder
- **Issue**: v1.0.1 encoder preserved underscores (`/claude_storage` → `-claude_storage`); Claude Code replaces them (`/claude_storage` → `-claude-storage`), causing project-not-found errors
- **Fix**: Encoder now replaces underscores with hyphens to match Claude Code behavior

### BUG-489: Session::stats() Hard-Failed on the First Malformed JSONL Line
- **File**: `stats_malformed_line_bug.rs`
- **Component**: `src/session.rs::stats()`
- **Issue**: A single syntactically-invalid JSONL line anywhere in a session file made `stats()` return `Err`, discarding all counts/tokens/timestamps accumulated from valid lines around it; sibling `load_entries()` reads the same file successfully
- **Fix**: Changed the per-line `parse_json(line).map_err(...)?` to `let Ok(json) = parse_json(line) else { continue; };`, mirroring `load_entries()`'s established silent-skip "Graceful Degradation Design"
- **Root Cause**: Hard `?`-propagation on a per-line parse error, inconsistent with the sibling function processing the same JSONL data and with the crate's own documented invariant (`docs/invariant/001_safety_guarantees.md`: "Malformed JSONL lines emit a warning and are skipped")

### BUG-491: is_agent_session()'s Doc Comment Claimed an isSidechain Check That Was Never Implemented
- **File**: `is_agent_session_doc_mismatch_bug.rs`
- **Component**: `src/session.rs::is_agent_session()`
- **Issue**: Doc comment claimed detection via filename prefix OR `isSidechain: true` in entries; code only ever checked the filename prefix — a reader could reasonably (and incorrectly) assume a non-`agent-`-prefixed session with sidechain entries would be detected
- **Fix**: Corrected the doc comment to describe only the actual filename-based check; no code logic changed — the implementation already matched the canonical algorithm (`docs/algorithm/001_agent_session_tracking.md`), which keeps filename-based `is_agent_session` and entry-based `is_agent_entry` deliberately separate
- **Root Cause**: The doc comment (present since the initial commit) described an aspirational second signal that was never implemented and never matched the canonical algorithm doc

### BUG-492: sessions_filtered() Discarded an Entire Project's Session List on One Corrupted Session
- **File**: `sessions_filtered_corrupted_session_bug.rs`
- **Component**: `src/project.rs::sessions_filtered()`
- **Issue**: `session.matches_filter( filter )?` hard-propagated a `Session::count_entries()` failure (e.g. a crash-truncated JSONL file failing UTF-8 validation) from `matches_filter()`'s `min_entries` branch, discarding every already-collected valid session in the project — not just the corrupted one
- **Fix**: Changed the loop to `match session.matches_filter( filter ) { Ok(true) => ..., Ok(false) => {}, Err(e) => eprintln!("Warning: ...") }`, mirroring the graceful per-session skip already used by `sessions()`, `all_sessions()`, and `project_stats()` in the same file
- **Root Cause**: `sessions_filtered()` was the sole outlier among 4 per-session loops in `project.rs` still using hard `?`-propagation instead of the file's own established catch-and-skip convention

## Related Documentation

- **Test Organization**: `test_organization.rulebook.md` — test documentation format standards
- **Codebase Hygiene**: `codebase_hygiene.rulebook.md` — quality standards for documentation
- **Invariants**: `../docs/invariant/` — known pitfalls and workspace-level constraints
