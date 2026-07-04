# Feature: Continuation Detection

### Scope

- **Purpose**: Provide callers with a reliable way to detect whether a Claude Code session exists for a given working directory and to identify the most-recently-used session by UUID.
- **Responsibility**: Documents the CWD → encoded storage path → UUID selection algorithm, the public API surface, and the filtering rules that distinguish user conversation files from agent definitions and empty initialization artifacts.
- **In Scope**: `check_continuation()`, `most_recent_session_id()`, `most_recent_session_in_dir()`, `to_storage_path_for()`, path encoding lookup, file inclusion/exclusion rules, mtime-based UUID selection.
- **Out of Scope**: Path encoding algorithm (→ `algorithm/001_path_encoding.md`), session entry parsing (→ `data_structure/001_storage_hierarchy.md`), caller-side retry or mismatch detection logic (→ `claude_runner` crate).

### Design

Claude Code v2.0+ stores conversation files in `~/.claude/projects/{encoded}/` where `{encoded}` is the lossy v1 encoding of the CWD (see `algorithm/001_path_encoding.md`). Continuation detection answers the question: "Does a prior session exist for this working directory, and if so, which one?"

**Two-level API:**

The implementation is factored into two levels to support callers with different needs:

- **Lower level** — `most_recent_session_in_dir(storage_path: &Path) -> Option<SessionId>`: operates on an already-resolved storage directory. Callers that know the storage path directly (e.g. from a custom `--session-dir`) call this without path encoding.
- **Upper level** — `most_recent_session_id(session_dir: &Path) -> Option<SessionId>`: encodes `session_dir` via `to_storage_path_for()`, then delegates to `most_recent_session_in_dir()`. Callers that work with the CWD or project directory call this.
- **Boolean convenience** — `check_continuation(session_dir: &Path) -> bool`: independent implementation that also detects legacy `conversation.json` and `.claude*` files that `most_recent_session_in_dir()` intentionally excludes. Preserved for callers that only need existence detection and must handle pre-v2 session formats. NOT a delegate to `most_recent_session_id().is_some()` — doing so would silently drop legacy format detection.

**UUID selection — most-recent mtime:**

When multiple `.jsonl` session files exist in the storage directory, `most_recent_session_in_dir()` returns the UUID of the file with the highest modification time. This matches Claude Code's own `-c`/`--continue` resolution: Claude resumes the most-recently-modified session when no explicit UUID is given.

**File inclusion/exclusion rules:**

A file counts as a resumable conversation session if it:
- Has a `.jsonl` extension (case-insensitive)
- Is NOT prefixed with `agent-` — these are agent/sub-conversation definitions, not user sessions
- Is NOT 0 bytes — Claude Code creates empty `.jsonl` files during initialization or after a crash; 0-byte files contain no conversation history and must be excluded to prevent false-positive detection

Non-`.jsonl` formats (`conversation.json`, `.claude*` files) detected by `check_continuation()` for legacy compatibility do NOT yield UUIDs — `most_recent_session_in_dir()` only returns UUIDs from `.jsonl` stems. Callers that need the UUID must use the new API; callers that only need the boolean can continue using `check_continuation()`.

**`SessionId` type:**

The return type of `most_recent_session_id()` and `most_recent_session_in_dir()` is `Option<SessionId>`. `SessionId` is an opaque newtype wrapping the UUID string extracted from the `.jsonl` filename stem. It implements `Display`, `AsRef<str>`, `From<String>`, `From<&str>`, `Clone`, `Eq`, and `Hash`.

### Algorithm

```text
most_recent_session_id(cwd):
  1. storage_path = to_storage_path_for(cwd)?           # encode cwd → ~/.claude/projects/{enc}/
  2. most_recent_session_in_dir(storage_path)

most_recent_session_in_dir(storage_path):
  1. read_dir(storage_path) → entries
  2. for each entry:
     a. skip if filename starts with "agent-"           # agent definitions, not user sessions
     b. skip if file is 0 bytes                         # initialization artifacts / crash remnants
     c. skip if extension is not ".jsonl" (case-insensitive)
     d. record (mtime, stem) for qualifying entries
  3. return SessionId::new(stem) for the entry with max mtime
     None if no qualifying entry found

check_continuation(cwd):
  1. storage_path = to_storage_path_for(cwd)?          # encode cwd → ~/.claude/projects/{enc}/
  2. read_dir(storage_path) → entries
  3. for each entry:
     a. skip if filename starts with "agent-"          # agent definitions, not user sessions
     b. skip if file is 0 bytes                        # initialization artifacts / crash remnants
     c. return true if extension is ".jsonl"           # v2+ session files (also yields UUIDs)
     d. return true if filename is "conversation.json" # legacy format (no UUID)
     e. return true if filename starts with ".claude"  # legacy format (no UUID)
  4. return false                                      # no qualifying file found
  # NOTE: check_continuation() is NOT most_recent_session_id().is_some() —
  #   the extra legacy format detection (d, e) makes it deliberately independent.
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/continuation.rs` | Full implementation |
| source | `../../src/session_id.rs` | `SessionId` newtype definition |
| doc | `../algorithm/001_path_encoding.md` | CWD → storage directory path encoding |
| doc | `../api/001_public_api.md` | Public API surface including continuation functions |
| doc | `../data_structure/001_storage_hierarchy.md` | Session file structure and JSONL format |

### Sources

| File | Notes |
|------|-------|
| `../../src/continuation.rs` | `check_continuation`, `most_recent_session_id`, `most_recent_session_in_dir`, `to_storage_path_for` |
| `../../tests/continuation_tests.rs` | Integration tests using temp HOME override and real filesystem |

### Tests

| File | Notes |
|------|-------|
| `../../tests/continuation_tests.rs` | `check_continuation_*` cases + new `most_recent_session_in_dir_*` cases |
| `../../tests/session_id_tests.rs` | `SessionId` newtype unit tests |
