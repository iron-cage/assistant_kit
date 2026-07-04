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
| **Type** | `DirectoryPath` — must be an existing directory |
| **Default** | absent (no cross-loading; uses target dir's own session) |
| **Env var** | `CLR_SESSION_FROM` |
| **Group** | Runner Control (`02_runner_control.md`) |
| **Commands** | `run`, `ask` |

### Behavior

When `--session-from <DIR>` is given:

1. `scope_for(DIR)` is called to compute `CLAUDE_SESSION_DIR` for `<DIR>`.
2. The most recently modified qualifying `.jsonl` file in that `CLAUDE_SESSION_DIR` is selected as the source session UUID.
3. `-c <uuid>` is injected into the claude subprocess arguments, causing Claude to resume from that session.
4. Claude runs in the **target** directory (`--dir` or CWD), not in `<DIR>`.
5. New conversation turns are written to the **target** directory's `CLAUDE_SESSION_DIR`.

This is a one-time cross-load. The source directory's session is not modified.

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
