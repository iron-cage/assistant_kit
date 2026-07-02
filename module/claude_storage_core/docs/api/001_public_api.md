# API: Public API

### Scope

- **Purpose**: Define the stable public interface contract for consumers of the `claude_storage_core` library.
- **Responsibility**: Documents public types, their operations, error handling approach, and stability guarantees.
- **In Scope**: Public types and functions, error type, versioning policy, what is stable vs. subject to change.
- **Out of Scope**: Internal implementation (→ `algorithm/`, `data_structure/`), CLI interface (→ `claude_storage` crate).

### Abstract

The public API exposes the storage hierarchy (Storage, Project, Session, Entry), filter types (SessionFilter, ProjectFilter, StringMatcher), content search (SearchFilter, SearchMatch), export (ExportFormat, export_session), path utilities (encode_path, decode_path), and a JSON value type (JsonValue, parse_json). All fallible operations return `Result<T, Error>` with structured error variants.

### Operations

**Storage hierarchy access:**
- Construct `Storage` from a path — provides `list_projects()` and `list_projects_filtered()`.
- Load sessions from a `Project` — `sessions()` and `sessions_filtered()`.
- Read entries from a `Session` — `entries()` (full parse) or `count_entries()` (fast byte-level count).
- Append a new entry — `Session::append_entry()` (atomic, append-only).
- Statistics — `stats()` on Session, Project, or Storage.

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

**Session identifier:**
- `SessionId` — opaque newtype wrapping the UUID string from a `.jsonl` filename stem. Implements `Display`, `AsRef<str>`, `From<String>`, `From<&str>`, `Clone`, `PartialEq`, `Eq`, and `Hash`. Use `as_str()` for raw string access.

**JSON parsing:**
- `parse_json(input) -> Result<JsonValue>` — parse arbitrary JSON into a value tree.

### Error Handling

All I/O operations return `Result<T, Error>`. Error variants cover: `Io` (filesystem errors with path context), `Parse` (malformed JSONL with position context), and others. Corrupted JSONL lines are treated as warnings and skipped rather than hard errors, enabling graceful degradation on partially-corrupted sessions.

Consumers should match on `Error` variants for structured handling rather than converting to string. Error messages include contextual path information for diagnosability.

### Compatibility Guarantees

**Backward compatibility is a non-goal.** All public types, function signatures, error variants, and JSON value representation may change between versions. There are no stability guarantees for this library — dependent crates must update simultaneously with the library. This policy enables clean evolution to match changes in Claude Code's storage format.

Major version bumps are used for breaking changes. A changelog entry is required for every breaking change.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/lib.rs` | Public API re-exports |
| source | `../../src/error.rs` | Error type definition |
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
