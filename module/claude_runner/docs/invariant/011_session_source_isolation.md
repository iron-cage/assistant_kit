# Invariant: Session Source Isolation

### Scope

- **Purpose**: Define the isolation contract for `--from` — which directory owns session reads vs. session writes during cross-loading.
- **Responsibility**: State that session reads come from the source directory's `CLAUDE_SESSION_DIR`, while Claude runs in (and writes to) the target directory's `CLAUDE_SESSION_DIR`.
- **In Scope**: `--from` read/write isolation, target-dir ownership of new session data, one-time-load semantics, `--session-dir`'s deprecation (inert, never suppresses `--from`), default-to-cwd interaction with the self-copy guard.
- **Out of Scope**: `scope_for()` internals (→ `../feature/005_session_path_resolution.md`); session file selection algorithm (→ `../algorithm/003_session_file_selection.md`); `--session-dir` raw path override behavior (→ `../cli/param/010_session_dir.md`).

### Invariant Statement

When `--from <SOURCE_DIR>` is given (or defaulted to CWD):

1. **Session reads use the source directory's storage.** The runner's continue decision (bare `-c` injection) is gated on a qualifying session existing in `scope_for(SOURCE_DIR).claude_session_dir` — NOT in the target directory's `CLAUDE_SESSION_DIR` — and that source session file is physically copied into the target's own storage before spawn (never the reverse), so claude's `-c` continues the transplanted history. No target-derived path is ever used as a transplant source.

2. **Claude runs in the target directory.** The subprocess working directory is set to `--dir` (or CWD if `--dir` is absent). The target directory is unchanged.

3. **New session data is written to the target directory's storage.** Any conversation turns that Claude adds during the session append to the transplanted copy in the target directory's `CLAUDE_SESSION_DIR` (claude derives that storage from the subprocess `HOME` + working directory — both target-side). The source directory's session files are never written to. On the destination side, an existing non-empty copy of the same session is overwritten by default (an explicit `--from` re-clones fresh, announced on stderr); `--keep-clone` preserves it (mtime refresh only) — either way, only target-side storage is touched, so the source-isolation invariant holds unchanged.

4. **Cross-loading is one-time, not persistent.** After the initial `-c` injection, the session evolves independently of the source. There is no ongoing mirroring or sync between source and target.

5. **`--session-dir` is deprecated and inert.** claude ≥2.x ignores the `CLAUDE_CODE_SESSION_DIR` export it used to trigger, for both reads and writes (BUG-490) — so it never suppresses `--from`'s transplant, and its raw path never appears in the assembled command. Setting `--session-dir`/`CLR_SESSION_DIR` to a non-empty value emits a deprecation warning naming the value; `--from` is the only mechanism that still works for cross-loading.

**Default-to-cwd interaction.** `--from` defaults to CWD when omitted (same default as `--to`). When source and target both resolve to the same storage — including the bare-invocation case where neither flag is given — the self-copy guard suppresses the transplant plan entirely; this invariant's isolation guarantees apply only when source and target storage differ.

### Enforcement

| Layer | Enforcement Mechanism |
|-------|-----------------------|
| `src/cli/builder.rs` | `session_exists(session_from_dir, effective_dir)` always receives the storage computed from `--from` (defaults to CWD when omitted/empty) — `--session-dir` is never consulted (BUG-493 dropped it from this lookup entirely); session reads always resolve to source storage, never target |
| `src/cli/builder.rs` | `build_claude_command()` computes `session_from_dir` unconditionally from `--from` (or its CWD default); the source value is canonicalized to its physical absolute form (and empty values ignored) before encoding — `--session-dir` cannot suppress this computation |
| `src/cli/builder.rs` | `execute_session_transplant()` copies the source `<uuid>.jsonl` into `scope_for(target).claude_session_dir` before spawn — the read-side steering mechanism. Overwrites an existing non-empty destination by default (announced, `--quiet`-gated); `--keep-clone` preserves it (mtime refresh only); self-copy (source storage == target storage) plans nothing; a failed copy warns (`[Runner] warning:`) and proceeds |
| `src/cli/mod.rs` | `dispatch_run` executes the transplant after working-dir validation, before journal + spawn; `--dry-run` previews the plan as `# session-transplant: <src> -> <dst>` without copying |
| Claude subprocess | Runs with the target directory as working directory; runner-side reads of the source are verified non-mutating (`session_from_test.rs::us7_source_session_files_not_modified`, `session_source_isolation_test.rs::in3_source_session_file_unchanged`) |

**Enforcement history.** An earlier mechanism exported `CLAUDE_CODE_SESSION_DIR=<source storage>` to steer claude's session selection at the source, leaving statements 1 and 3 dependent on claude honoring that variable (contract [B23](../../../../contract/claude_code/docs/behavior/023_b23_session_dir_override.md), NEG-ONLY). BUG-490 established empirically that claude 2.x ignores the variable for both reads and writes — the redirect was inert and `--from` a silent no-op. The physical transplant replaced it: statements 1–3 are now runner-enforced by construction (the session file is placed where claude's own cwd-derived storage lookup will find it), with degradation on copy failure limited to a fresh session in the target — never a write to the source.

### Violation Consequences

If the invariant is broken:

- **Write-to-source violation:** Session data from the cross-loaded run accumulates in the source directory's storage, polluting it with unrelated conversation history.
- **Read-from-target violation:** `--from` becomes a no-op (equivalent to default behavior), making cross-loading impossible.
- **Deprecation-silence violation:** `--session-dir`/`CLR_SESSION_DIR` set to a non-empty value fails to emit its deprecation warning, leaving users unaware the override has no effect on session storage.

### Related Docs

| File | Relationship |
|------|--------------|
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` contract and cross-loading scenarios |
| [`../algorithm/003_session_file_selection.md`](../algorithm/003_session_file_selection.md) | Session file selection — how the source session UUID is determined |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — the variable computed for both source and target |
| [`../cli/param/010_session_dir.md`](../cli/param/010_session_dir.md) | `--session-dir` — deprecated, inert raw override (BUG-493); never suppresses `--from` |
| [`../cli/param/076_from.md`](../cli/param/076_from.md) | `--from` parameter reference |
