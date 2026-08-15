# Invariant: Session Source Isolation

### Scope

- **Purpose**: Define the isolation contract for `--session-from` — which directory owns session reads vs. session writes during cross-loading.
- **Responsibility**: State that session reads come from the source directory's `CLAUDE_SESSION_DIR`, while Claude runs in (and writes to) the target directory's `CLAUDE_SESSION_DIR`.
- **In Scope**: `--session-from` read/write isolation, target-dir ownership of new session data, one-time-load semantics, precedence over `--session-dir`.
- **Out of Scope**: `scope_for()` internals (→ `../feature/005_session_path_resolution.md`); session file selection algorithm (→ `../algorithm/003_session_file_selection.md`); `--session-dir` raw path override behavior (→ `../cli/param/010_session_dir.md`).

### Invariant Statement

When `--session-from <SOURCE_DIR>` is given:

1. **Session reads use the source directory's storage.** The runner's continue decision (bare `-c` injection) is gated on a qualifying session existing in `scope_for(SOURCE_DIR).claude_session_dir` — NOT in the target directory's `CLAUDE_SESSION_DIR` — and the subprocess receives `CLAUDE_CODE_SESSION_DIR=<source storage>` so claude's own session selection is pointed at the source.

2. **Claude runs in the target directory.** The subprocess working directory is set to `--dir` (or CWD if `--dir` is absent). The target directory is unchanged.

3. **New session data is written to the target directory's storage.** Any conversation turns that Claude adds during the session are written to the target directory's `CLAUDE_SESSION_DIR` (controlled by Claude Code itself, based on the subprocess `HOME` + working directory). The source directory's session files are never written to.

4. **Cross-loading is one-time, not persistent.** After the initial `-c` injection, the session evolves independently of the source. There is no ongoing mirroring or sync between source and target.

5. **`--session-dir` takes precedence.** If both `--session-from` and `--session-dir` are given, `--session-dir` (raw path) wins. `--session-from` is a higher-level convenience that computes the source storage path; `--session-dir` bypasses that computation entirely.

### Enforcement

| Layer | Enforcement Mechanism |
|-------|-----------------------|
| `src/cli/builder.rs` | `session_exists(session_dir, effective_dir)` uses `scope_for(source_dir).claude_session_dir` as the storage path when `--session-from` is set; falls back to `scope_for(effective_dir).claude_session_dir` otherwise |
| `src/cli/builder.rs` | `build_claude_command()` checks `--session-dir` first; `--session-from` is consulted only when `--session-dir` is absent; the source value is canonicalized to its physical absolute form (and empty values ignored) before encoding |
| `ClaudeCommand` env pairs | The subprocess receives `CLAUDE_CODE_SESSION_DIR=<source storage>` ([contract B23](../../../../contract/claude_code/docs/behavior/023_b23_session_dir_override.md)) — the read-side steering mechanism |
| Claude subprocess | Runs with the target directory as working directory; runner-side reads of the source are verified non-mutating (`session_from_test.rs::us7_source_session_files_not_modified`) |

**Enforcement caveat — write side is claude-dependent.** Statements 2 and 5 are runner-enforced. Statements 1 and 3, at runtime, rest on how claude treats `CLAUDE_CODE_SESSION_DIR`, whose contract status is NEG-ONLY (verified not rejected at startup; not confirmed honored — see contract B23). If claude honors the variable for both reads and writes, new turns land in the **source** storage, violating statement 3; if claude ignores it, session selection falls back to the target's own storage, weakening statement 1. Neither failure mode is currently detectable from the runner: resolving this requires upgrading B23's evidence tier (a VALIDATED live observation of where a cross-loaded turn is actually written).

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
