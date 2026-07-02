# Variable: CLAUDE_SESSION_FILE

### Scope

- **Purpose**: Document the most recently active session file path — the `.jsonl` conversation file that `claude -c` would resume.
- **Responsibility**: Define the value format, derivation rule (mtime-based selection), empty-string sentinel, and examples for `CLAUDE_SESSION_FILE`.
- **In Scope**: CLAUDE_SESSION_FILE derivation from CLAUDE_SESSION_DIR, qualifying file criteria, empty-string convention.
- **Out of Scope**: CLAUDE_SESSION_DIR derivation (→ `003_claude_session_dir.md`); session file selection algorithm details (→ `../algorithm/003_session_file_selection.md`); what the file contains.

### Value Format

- **With active session:** Absolute filesystem path ending in `/{uuid}.jsonl`. The UUID stem matches the session identifier used by `claude -c`.
- **No session history:** Empty string (`""`).

**Examples:**
- `/home/alice/.claude/projects/-home-alice-project/9a3f8a12-cdef-4567-8901-abcdef012345.jsonl`
- `` (empty — no sessions yet)

### Derivation

```
CLAUDE_SESSION_FILE = highest_mtime_qualifying_jsonl(CLAUDE_SESSION_DIR)
                    | ""  if none exists
```

**Qualifying file criteria** (a file must satisfy ALL to be considered):
1. Extension is `.jsonl` (case-insensitive).
2. Filename does NOT begin with `agent-`.
3. File size > 0 bytes.

The file with the highest modification time among all qualifying files is selected. Its full path is `CLAUDE_SESSION_DIR / filename`.

See `../algorithm/003_session_file_selection.md` for the complete algorithm.

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | none |

`CLAUDE_SESSION_FILE` is purely read-only output from `scope_for()`. To inject a specific session, use `--session-dir` (raw path) or `--session-from <DIR>` (directory whose session is computed via `scope_for()`).

### Examples

| CLAUDE_SESSION_DIR | Qualifying files | CLAUDE_SESSION_FILE |
|--------------------|------------------|---------------------|
| `-home-alice-project/` | `abc.jsonl` (T1), `def.jsonl` (T2, T2>T1) | `…/-home-alice-project/def.jsonl` |
| `-home-alice-project/` | none | `` (empty) |
| `-home-alice-project/` | `agent-tools.jsonl` only | `` (filtered out) |
| `-home-alice-project/` | `session.jsonl` (0 bytes) | `` (zero-byte excluded) |

### Related Docs

| File | Relationship |
|------|--------------|
| [`003_claude_session_dir.md`](003_claude_session_dir.md) | CLAUDE_SESSION_DIR — the directory scanned to produce CLAUDE_SESSION_FILE |
| [`../algorithm/003_session_file_selection.md`](../algorithm/003_session_file_selection.md) | Session file selection algorithm — the exact scan and filter steps |
| [`../cli/param/010_session_dir.md`](../cli/param/010_session_dir.md) | `--session-dir` — raw override for session storage path |
| [`../cli/param/076_session_from.md`](../cli/param/076_session_from.md) | `--session-from` — computes CLAUDE_SESSION_DIR for source dir |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |
