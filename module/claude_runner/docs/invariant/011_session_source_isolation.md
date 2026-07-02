# Invariant: Session Source Isolation

### Scope

- **Purpose**: Define the isolation contract for `--session-from` — which directory owns session reads vs. session writes during cross-loading.
- **Responsibility**: State that session reads come from the source directory's `CLAUDE_SESSION_DIR`, while Claude runs in (and writes to) the target directory's `CLAUDE_SESSION_DIR`.
- **In Scope**: `--session-from` read/write isolation, target-dir ownership of new session data, one-time-load semantics, precedence over `--session-dir`.
- **Out of Scope**: `scope_for()` internals (→ `../feature/005_session_path_resolution.md`); session file selection algorithm (→ `../algorithm/003_session_file_selection.md`); `--session-dir` raw path override behavior (→ `../cli/param/010_session_dir.md`).

### Invariant Statement

When `--session-from <SOURCE_DIR>` is given:

1. **Session reads use the source directory's storage.** The session UUID injected via `-c` is selected from `scope_for(SOURCE_DIR).claude_session_dir` — NOT from the target directory's `CLAUDE_SESSION_DIR`.

2. **Claude runs in the target directory.** The subprocess working directory is set to `--dir` (or CWD if `--dir` is absent). The target directory is unchanged.

3. **New session data is written to the target directory's storage.** Any conversation turns that Claude adds during the session are written to the target directory's `CLAUDE_SESSION_DIR` (controlled by Claude Code itself, based on the subprocess `HOME` + working directory). The source directory's session files are never written to.

4. **Cross-loading is one-time, not persistent.** After the initial `-c <uuid>` injection, the session evolves in the target directory's storage. There is no ongoing mirroring or sync between source and target.

5. **`--session-dir` takes precedence.** If both `--session-from` and `--session-dir` are given, `--session-dir` (raw path) wins. `--session-from` is a higher-level convenience that computes the source storage path; `--session-dir` bypasses that computation entirely.

### Enforcement

| Layer | Enforcement Mechanism |
|-------|-----------------------|
| `src/cli/builder.rs` | `session_exists(session_dir, effective_dir)` uses `scope_for(source_dir).claude_session_dir` as the storage path when `--session-from` is set; falls back to `scope_for(effective_dir).claude_session_dir` otherwise |
| `src/cli/builder.rs` | `build_claude_command()` checks `--session-dir` first; `--session-from` is consulted only when `--session-dir` is absent |
| Claude subprocess | Runs with `--dir <target_dir>` as working directory; all new session writes are directed by Claude Code to the target's storage — no runner-level enforcement needed |

### Violation Consequences

If the invariant is broken:

- **Write-to-source violation:** Session data from the cross-loaded run accumulates in the source directory's storage, polluting it with unrelated conversation history.
- **Read-from-target violation:** `--session-from` becomes a no-op (equivalent to default behavior), making cross-loading impossible.
- **Precedence violation:** `--session-dir` would be silently overridden by `--session-from`, breaking raw-path session injection workflows.

### Related Docs

| File | Relationship |
|------|--------------|
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` contract and cross-loading scenarios |
| [`../algorithm/003_session_file_selection.md`](../algorithm/003_session_file_selection.md) | Session file selection — how the source session UUID is determined |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — the variable computed for both source and target |
| [`../cli/param/010_session_dir.md`](../cli/param/010_session_dir.md) | `--session-dir` — raw override; takes precedence over `--session-from` |
| [`../cli/param/076_session_from.md`](../cli/param/076_session_from.md) | `--session-from` parameter reference |
