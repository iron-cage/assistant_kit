# Parameter: --session-from

### Scope

- **Purpose**: Document the `--session-from <DIR>` parameter — session cross-loading source directory.
- **Responsibility**: Specify the type, default, derivation behavior, alias, interaction with `--session-dir`, and usage examples.
- **In Scope**: `--session-from` semantics, `--from` alias, precedence vs `--session-dir`, applicable commands.
- **Out of Scope**: `scope_for()` internals (→ `../feature/005_session_path_resolution.md`); isolation contract (→ `../invariant/011_session_source_isolation.md`); `--dir`/`--to` (→ `008_dir.md`).

### Definition

| Attribute | Value |
|-----------|-------|
| **Flag** | `--session-from <DIR>` |
| **Short alias** | `--from <DIR>` |
| **Type** | `DirectoryPath` — resolved to physical absolute form; need not exist (a nonexistent source has no sessions → no cross-load, fresh session) |
| **Path resolution** | `fs::canonicalize` (symlinks + `..` resolved); lexical cwd-join fallback for nonexistent paths |
| **Empty value** | ignored entirely — same empty-is-identity rule as `--subdir ""` |
| **Default** | absent (no cross-loading; uses target dir's own session) |
| **Env var** | `CLR_SESSION_FROM` |
| **Config key** | `session-from` (args-file JSON) |
| **Group** | Runner Control (`02_runner_control.md`) |
| **Commands** | `run`, `ask` |

### Behavior

When `--session-from <DIR>` is given:

1. `<DIR>` is resolved to its physical absolute form — claude derives storage names from its physical getcwd, so an unresolved relative or symlinked value would encode to a storage name claude never uses (`./src` → `---src`). An empty value is ignored entirely.
2. `scope_for(resolved DIR)` computes the source `CLAUDE_SESSION_DIR`.
3. The runner checks that storage dir for the most recently modified qualifying `.jsonl` (see `../algorithm/003_session_file_selection.md`).
4. If one exists, bare `-c` (continue) is injected into the subprocess arguments — no UUID is passed on the command line; session selection inside claude is steered by the physical transplant below.
5. The source session file is physically copied into the **target's own** storage dir (`scope_for(target).claude_session_dir`) before spawn, so plain `-c` continues the transplanted history in place under the same UUID. If source and target resolve to the same storage dir, no transplant is planned (the session is already in place). If a file with the same name already exists in target storage, it is never overwritten — only its mtime is refreshed so `-c` selects it. A failed copy warns loudly (`[Runner] warning:`) and proceeds, degrading to a fresh session. Under `--dry-run` no copy happens; the plan is previewed as `# session-transplant: <src_file> -> <target_storage_dir>`. (The former mechanism — exporting `CLAUDE_CODE_SESSION_DIR=<source storage>` — is inert on claude 2.x, which ignores that variable for both reads and writes; see BUG-490. [Contract B23](../../../../../contract/claude_code/docs/behavior/023_b23_session_dir_override.md)'s NEG-ONLY grading anticipated exactly this: "not rejected at startup" never implied "honored".)
6. Claude runs in the **target** directory (`--dir`/`--to` or CWD), not in `<DIR>`.

This is a one-time cross-load; the runner reads the source directory's session files but never modifies them — the transplant is a copy outward. See `../../invariant/011_session_source_isolation.md` for the read/write isolation contract.

**Higher-level than `--session-dir`:**
- `--session-dir /path` takes the raw path verbatim as the session storage directory.
- `--session-from /home/alice/project` computes `Df("/home/alice/project")` and uses `~/.claude/projects/-home-alice-project` as the session storage path. Ergonomically equivalent but requires only the project directory, not the encoded storage path.

**Precedence:** If both `--session-from` and `--session-dir` are given, `--session-dir` takes precedence (raw path wins over computed path).

### Usage

```sh
# Run in CWD but use session from /home/alice/project-a
clr "Continue this work" --session-from /home/alice/project-a

# Alias form
clr "Continue" --from /home/alice/project-a

# Clone outward: run in project-b, use session from project-a
clr --to /home/alice/project-b --session-from /home/alice/project-a "Adapt this feature"

# Inject inward: run in project-a, query session from project-b
clr --session-from /home/alice/project-b "What did you implement in B?"

# Env var form
CLR_SESSION_FROM=/home/alice/project-a clr "Continue"
```

### Interaction with Other Parameters

| Parameter | Interaction |
|-----------|-------------|
| `--session-dir` | `--session-dir` takes precedence; `--session-from` ignored when both given |
| `--dir` / `--to` | `--dir`/`--to` sets where Claude runs; `--session-from` sets where the session is loaded from — they are independent |
| `--new-session` | `--new-session` suppresses `-c` injection; if both given, `--new-session` wins (no session loaded) |
| `--session-from` (no session history) | If the source dir has no qualifying session files, no `-c` is injected (no cross-loading occurs; Claude starts fresh in target dir) |

### Related Parameters

| # | Parameter | Relationship |
|---|-----------|--------------|
| 010 | [`--session-dir`](010_session_dir.md) | Raw session storage path override; takes precedence over `--session-from` |
| 008 | [`--dir`](008_dir.md) | Target directory where Claude runs; `--to` is an alias |
| 007 | [`--new-session`](007_new_session.md) | Suppresses session continuation; takes precedence over `--session-from` |

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
