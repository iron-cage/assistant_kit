# Parameter: --from

### Scope

- **Purpose**: Document the `--from <DIR>` parameter — session cross-loading source directory.
- **Responsibility**: Specify the type, default-to-cwd behavior, interaction with `--session-dir`, and usage examples.
- **In Scope**: `--from` semantics, default-to-cwd rule (shared with `--to`/`--dir`), `--session-dir`'s deprecation (inert, never suppresses `--from`), applicable commands.
- **Out of Scope**: `scope_for()` internals (→ `../feature/005_session_path_resolution.md`); isolation contract (→ `../invariant/011_session_source_isolation.md`); `--dir`/`--to` (→ `008_dir.md`).

### Definition

| Attribute | Value |
|-----------|-------|
| **Flag** | `--from <DIR>` |
| **Type** | `DirectoryPath` — resolved to physical absolute form; need not exist (a nonexistent source has no sessions → no cross-load, fresh session) |
| **Path resolution** | `fs::canonicalize` (symlinks + `..` resolved); for nonexistent paths, the deepest existing prefix is canonicalized and the nonexistent tail appended literally, matching claude's own physical-getcwd key once the directory is created (Fix(BUG-543)) |
| **Empty value** | ignored entirely — same empty-is-identity rule as `--topic ""` |
| **Default** | when the target (`--dir`/`--to`/`--topic`, or its own default of cwd) already has a qualifying session of its own, that target IS the source — no re-derivation from cwd (Fix(BUG-541); see step 1 below); otherwise, current working directory — same default-to-cwd rule as `--dir`/`--to`. When both `--from` and `--to` are omitted, source and target resolve to the same storage, so the self-copy guard (below) suppresses the transplant and the run is an ordinary no-op |
| **Env var** | `CLR_FROM` |
| **Config key** | `from` (args-file JSON) |
| **Group** | Runner Control (`02_runner_control.md`) |
| **Commands** | `run`, `ask`, `topic` |

### Behavior

`--from <DIR>` (or its resolved default — see step 1) is compared against the target (`--to`/`--dir`, or its own default, cwd):

1. When `--from` is omitted or empty, `<DIR>` defaults to the target's own storage when that target (`--dir`/`--to`/`--topic`'s resolved effective directory) already has a qualifying session of its own — a repeat call against an already-established target continues that target's own history instead of re-deriving from cwd's possibly-since-changed most-recent session (Fix(BUG-541)). A bare invocation with no `--dir`/`--to`/`--topic` at all, or a target with no session yet (a genuine first use), falls back to cwd — both unaffected by the fix, since cwd was already the correct answer for them.
2. `<DIR>` (explicit `--from`, or whichever default step 1 selected) is resolved to its physical absolute form — claude derives storage names from its physical getcwd, so an unresolved relative or symlinked value would encode to a storage name claude never uses (`./src` → `---src`).
3. `scope_for(resolved DIR)` computes the source `CLAUDE_SESSION_DIR`; `scope_for(resolved target)` computes the target's.
4. **Self-copy guard**: if source and target storage resolve to the same directory (true whenever both `--from` and `--to` are omitted, or when they're explicitly given the same effective directory), no transplant is planned — the session is already in place, and ordinary continuation detection (bare `-c` when the target's own storage already has a qualifying session) applies unchanged.
5. Otherwise, the runner checks the source storage dir for the most recently modified qualifying `.jsonl` (see `../algorithm/003_session_file_selection.md`).
6. If one exists, bare `-c` (continue) is injected into the subprocess arguments — no UUID is passed on the command line; session selection inside claude is steered by the physical transplant below.
7. The source session file is physically copied into the **target's own** storage dir (`scope_for(target).claude_session_dir`) before spawn, so plain `-c` continues the transplanted history in place under the same UUID. If a file with the same name already exists in target storage, it is never overwritten — only its mtime is refreshed so `-c` selects it. A failed copy warns loudly (`[Runner] warning:`) and proceeds, degrading to a fresh session. Under `--dry-run` no copy happens; the plan is previewed as `# session-transplant: <src_file> -> <target_storage_dir>`. (The former mechanism — exporting `CLAUDE_CODE_SESSION_DIR=<source storage>` — is inert on claude 2.x, which ignores that variable for both reads and writes; see BUG-490. [Contract B23](../../../../../contract/claude_code/docs/behavior/023_b23_session_dir_override.md)'s NEG-ONLY grading anticipated exactly this: "not rejected at startup" never implied "honored".)
8. Claude runs in the **target** directory (`--dir`/`--to` or CWD), not in `<DIR>`.

This is a one-time cross-load; the runner reads the source directory's session files but never modifies them — the transplant is a copy outward. See `../../invariant/011_session_source_isolation.md` for the read/write isolation contract.

**`--session-dir` is deprecated and inert:**
- `--session-dir /path`/`CLR_SESSION_DIR` used to override the raw session storage directory via a `CLAUDE_CODE_SESSION_DIR` export; claude ≥2.x ignores that override entirely for both reads and writes (BUG-490), so setting it has no effect on where sessions load from or save to.
- `--from /home/alice/project` (via the physical transplant above) is the only mechanism that still works for cross-loading another project's session history.

**Precedence:** `--session-dir` never suppresses `--from` — its transplant proceeds exactly as if `--session-dir` were absent. Setting `--session-dir`/`CLR_SESSION_DIR` to a non-empty value emits a deprecation warning (unless `--quiet`) naming the value; see [`010_session_dir.md`](010_session_dir.md).

**No backward-compatible alias:** the pre-rename flag name `--session-from` (and its env var `CLR_SESSION_FROM`) is no longer recognized — a breaking rename, not an alias. `--session-from` now fails parsing with the standard unknown-option error.

### Usage

```sh
# Run in CWD but use session from /home/alice/project-a
clr "Continue this work" --from /home/alice/project-a

# Clone outward: run in project-b, use session from project-a
clr --to /home/alice/project-b --from /home/alice/project-a "Adapt this feature"

# Clone outward using only --to: --from defaults to cwd
clr --to /home/alice/project-b "Adapt this feature"

# Inject inward: run in project-a, query session from project-b
clr --from /home/alice/project-b "What did you implement in B?"

# Bare invocation: both default to cwd — self-copy guard makes this a no-op
clr "Continue"

# Env var form
CLR_FROM=/home/alice/project-a clr "Continue"
```

### Interaction with Other Parameters

| Parameter | Interaction |
|-----------|-------------|
| `--session-dir` | deprecated and inert (BUG-493); never suppresses `--from`'s transplant, emits a deprecation warning naming its value |
| `--dir` / `--to` | `--dir`/`--to` sets where Claude runs (also defaults to cwd); `--from` sets where the session is loaded from — independent flags that share the same default-to-cwd rule |
| `--new-session` | `--new-session` suppresses `-c` injection; if both given, `--new-session` wins (no session loaded) |
| `--from` (no session history) | If the source dir has no qualifying session files, no `-c` is injected (no cross-loading occurs; Claude starts fresh in target dir) |

### Related Parameters

| # | Parameter | Relationship |
|---|-----------|--------------|
| 010 | [`--session-dir`](010_session_dir.md) | Deprecated, inert raw storage override (BUG-493); never suppresses `--from` |
| 008 | [`--dir`](008_dir.md) | Target directory where Claude runs; `--to` is an alias; shares the same default-to-cwd rule as `--from` |
| 007 | [`--new-session`](007_new_session.md) | Suppresses session continuation; takes precedence over `--from` |

### Referenced Doc Instances

| File | Relationship |
|------|--------------|
| [`../feature/005_session_path_resolution.md`](../../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |
| [`../invariant/011_session_source_isolation.md`](../../invariant/011_session_source_isolation.md) | Isolation invariant: reads from source, writes to target |
| [`../variable/003_claude_session_dir.md`](../../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — computed for both source and target |
| [`../../algorithm/001_path_encoding.md`](../../algorithm/001_path_encoding.md) | Df() — applied to `<DIR>` to find its session storage |
| [`../algorithm/003_session_file_selection.md`](../../algorithm/003_session_file_selection.md) | Session selection — how the source session UUID is picked |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 28 | [028_session_transplant.md](../user_story/028_session_transplant.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |
