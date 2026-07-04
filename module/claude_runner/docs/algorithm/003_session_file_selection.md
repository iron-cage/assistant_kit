# Algorithm: Session File Selection

### Scope

- **Purpose**: Identify the most recently active conversation file in a Claude storage directory — replicating `claude -c` active-session choice.
- **Responsibility**: Specify the directory scan, file qualification filter, mtime comparison, and fallback behavior used to select `CLAUDE_SESSION_FILE`.
- **In Scope**: Directory listing, `.jsonl` extension filter, `agent-` prefix exclusion, zero-byte skip, mtime comparison, stem extraction.
- **Out of Scope**: Path encoding to reach the storage directory (→ `001_path_encoding.md`); git root detection (→ `002_git_root_detection.md`); the `CLAUDE_SESSION_FILE` variable definition (→ `../variable/006_claude_session_file.md`).

### Algorithm

Given a Claude storage directory `storage_path` (i.e., `CLAUDE_SESSION_DIR`):

1. Open `storage_path` for directory listing. If the directory does not exist or cannot be read: return `None`.
2. For each entry `E` in the directory:
   - **Skip** if `E.filename` starts with `agent-` (agent definition files, not conversations).
   - **Skip** if `E.extension` is not `.jsonl` (case-insensitive).
   - **Skip** if `E.size == 0` bytes (Claude Code initialization artifacts, empty after crash).
   - Record `(mtime, stem)` where `stem` = filename without the `.jsonl` extension.
3. From all recorded pairs, select the one with the **highest** `mtime`.
4. Return the full path: `storage_path / stem + ".jsonl"`.
5. If no qualifying entries were found: return `None` (empty string for `CLAUDE_SESSION_FILE`).

### Examples

| Storage Dir Contents | Result |
|----------------------|--------|
| `abc-123.jsonl` (5 min ago, 1 KB), `def-456.jsonl` (1 min ago, 2 KB) | `def-456.jsonl` (highest mtime) |
| `agent-tools.jsonl` only | `None` (all filtered by `agent-` prefix) |
| `session.jsonl` (0 bytes) only | `None` (zero-byte file excluded) |
| Empty directory | `None` |
| `old.jsonl` (yesterday), `new.jsonl` (1 sec ago) | `new.jsonl` |

**Full path example:**

```
CLAUDE_SESSION_DIR = ~/.claude/projects/-home-alice-project/
qualifying files   = 9a3f8a12-cdef-4567-8901-abcdef012345.jsonl (mtime: T1)
                     bb4c9e01-abcd-1234-5678-fedcba987654.jsonl (mtime: T2, T2 > T1)

CLAUDE_SESSION_FILE = ~/.claude/projects/-home-alice-project/bb4c9e01-abcd-1234-5678-fedcba987654.jsonl
```

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| `agent-*.jsonl` files | Excluded — these are agent definition files, not user conversations |
| 0-byte `.jsonl` file | Excluded — Claude Code creates empty files during initialization; they must not be treated as sessions |
| Non-`.jsonl` files (e.g., `settings.json`) | Excluded by extension filter |
| `CLAUDE_SESSION_DIR` does not exist | `None` — no history yet |
| Multiple qualifying files with identical mtime | Tie-breaking is implementation-defined; typically whichever the OS `read_dir` iterator returns last |

### Implementation

| Location | Symbol | Role |
|----------|--------|------|
| `claude_storage_core/src/continuation.rs` | `most_recent_session_in_dir(storage_path: &Path) -> Option<SessionId>` | Core scan returning UUID stem as `SessionId` |
| `claude_storage_core/src/continuation.rs` | `most_recent_session_id(session_dir: &Path) -> Option<SessionId>` | Encodes `session_dir` via Df() then delegates to `most_recent_session_in_dir` |
| `claude_storage_core/src/scope.rs` | `scope_for(dir: &Path) -> ClaudeScope` | Calls `most_recent_session_in_dir()` and reconstructs the full path as `storage_path.join(format!("{}.jsonl", id.as_str()))` for `claude_session_file` |

### Related Docs

| File | Relationship |
|------|--------------|
| [`001_path_encoding.md`](001_path_encoding.md) | Df() — encodes the working directory to reach `CLAUDE_SESSION_DIR` |
| [`002_git_root_detection.md`](002_git_root_detection.md) | Git root detection — parallel algorithm for memory path |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — the storage directory scanned by this algorithm |
| [`../variable/006_claude_session_file.md`](../variable/006_claude_session_file.md) | CLAUDE_SESSION_FILE — the output of this algorithm |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |
