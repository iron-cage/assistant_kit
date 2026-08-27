# API: Public API

### Scope

- **Purpose**: Define the stable public interface contract for consumers of the `claude_storage_core` library.
- **Responsibility**: Documents public types, their operations, error handling approach, and stability guarantees.
- **In Scope**: Public types and functions, error type, versioning policy, what is stable vs. subject to change.
- **Out of Scope**: Internal implementation (→ `algorithm/`, `data_structure/`), CLI interface (→ `claude_storage` crate).

### Abstract

The public API exposes the storage hierarchy (Storage, Project, Session, Entry), filter types (SessionFilter, ProjectFilter, StringMatcher), content search (SearchFilter, SearchMatch), export (ExportFormat, export_session), a token-usage rollup engine (GroupKey, SortKey, SortOrder, RollupInput, RollupRow, RollupParams, build_rollup), path utilities (encode_path, decode_path), and a JSON value type (JsonValue, parse_json). All fallible operations return `Result<T, Error>` with structured error variants.

### Operations

**Storage hierarchy access:**
- Construct `Storage` from a path — provides `list_projects()` and `list_projects_filtered()`.
- Load sessions from a `Project` — `sessions()` and `sessions_filtered()`.
- Read entries from a `Session` — `entries()` (full parse) or `count_entries()` (fast byte-level count).
- Append a new entry — `Session::append_entry()` (atomic, append-only).
- Statistics — `stats()` on Session, Project, or Storage.

**Token-usage rollup:**
- Assemble `RollupInput` values (one per session) from already-computed `SessionStats`.
- Call `build_rollup(entries, &RollupParams) -> Vec<RollupRow>` — pure aggregation, no I/O, cannot fail.
- `RollupParams` selects `group_by: GroupKey` (Session/Project/Model/Day), `sort_by: SortKey` (Total/Input/Output/Cache/MaxContext/Calls/Sessions/Group), `order: SortOrder` (Asc/Desc), an optional `model_filter: StringMatcher`, and a `limit` row cap.
- Each `RollupRow` reports `sessions`, `calls`, per-category token sums, `max_context`, `percent` (share of the full filtered grand total, computed before `limit` truncates), and `first`/`last` timestamps; `RollupRow::cache()`/`total()` combine fields.

**Content search:**
- Build a `SearchFilter` with query, case-sensitivity, optional role and content-type constraints.
- Invoke `Session::search()` or `Storage::search_all()` to get `SearchMatch` results.

**Export:**
- Select `ExportFormat` (Markdown, JSON, or Text).
- Invoke `export_session()` with a writer, or `export_session_to_file()` with an output path.

**Path utilities:**
- `encode_path(path) -> String` — encode a filesystem path as a storage directory name.
- `decode_path(encoded) -> Result<PathBuf>` — decode a storage directory name back to a path.

**Continuation detection:**
- `check_continuation(session_dir: &Path) -> bool` — returns `true` when non-empty, non-agent conversation files exist for the given working directory.
- `most_recent_session_id(session_dir: &Path) -> Option<SessionId>` — encodes `session_dir`, scans `~/.claude/projects/{encoded}/`, and returns the `SessionId` of the most-recently-modified qualifying `.jsonl` file.
- `most_recent_session_in_dir(storage_path: &Path) -> Option<SessionId>` — lower-level variant: operates directly on an already-resolved storage directory without path encoding. Used when the caller has a custom session directory.
- `to_storage_path_for(session_dir: &Path) -> Option<PathBuf>` — compute the Claude storage directory for a CWD without scanning it.

**Transcript answer reading:**
- `transcript_path(cwd: &Path, session_id: &str) -> Option<PathBuf>` — name the transcript for a conversation id without checking that it exists yet.
- `transcript_mark(path: &Path) -> usize` — how many conversation entries the transcript holds right now; a missing file is 0, not an error. Taken before a prompt is sent, so everything past it is that turn.
- `transcript_answer_since(path: &Path, mark: usize, grace: Duration) -> Option<String>` — the assistant's text written past `mark`, blocking up to `grace` for a transcript still being flushed. Text blocks only — thinking blocks, tool calls, and tool results are excluded. `None` means "nothing to show from here", not "the session said nothing".

**Session identifier:**
- `SessionId` — opaque newtype wrapping the UUID string from a `.jsonl` filename stem. Implements `Display`, `AsRef<str>`, `From<String>`, `From<&str>`, `Clone`, `PartialEq`, `Eq`, and `Hash`. Use `as_str()` for raw string access.

**JSON parsing:**
- `parse_json(input) -> Result<JsonValue>` — parse arbitrary JSON into a value tree.

### Error Handling

All I/O operations return `Result<T, Error>`. Error variants cover: `Io` (filesystem errors with path context), `Parse` (malformed JSONL with position context), and others. A malformed JSONL line within an otherwise-readable session file is silently skipped rather than surfaced as a hard error or a warning, enabling graceful degradation on partially-corrupted sessions — a whole-file read failure still returns `Err` (BUG-507).

Consumers should match on `Error` variants for structured handling rather than converting to string. Error messages include contextual path information for diagnosability.

### Compatibility Guarantees

**Backward compatibility is a non-goal.** All public types, function signatures, error variants, and JSON value representation may change between versions. There are no stability guarantees for this library — dependent crates must update simultaneously with the library. This policy enables clean evolution to match changes in Claude Code's storage format.

Major version bumps are used for breaking changes. A changelog entry is required for every breaking change.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/lib.rs` | Public API re-exports |
| source | `../../src/error.rs` | Error type definition |
| source | `../../src/rollup.rs` | GroupKey, SortKey, SortOrder, RollupInput, RollupRow, RollupParams, build_rollup() |
| source | `../../src/transcript_answer.rs` | transcript_path(), transcript_mark(), transcript_answer_since() |
| doc | `../feature/005_token_usage_rollup.md` | Rollup engine design rationale |
| doc | `../feature/006_transcript_answer.md` | Transcript answer reading design and algorithm |
| doc | `../data_structure/001_storage_hierarchy.md` | Storage, Project, Session, Entry types |
| doc | `../data_structure/002_filter_types.md` | Filter types |
| doc | `../feature/004_continuation_detection.md` | Continuation detection API design and algorithm |
| doc | `../feature/002_content_search.md` | Search API design |
| doc | `../feature/003_export_formats.md` | Export API design |
| doc | `../algorithm/001_path_encoding.md` | Path utility functions |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; public API and API stability sections extracted here |
