# CLI Parameter: --keep-clone

Opt-out flag for the session-transplant re-clone default. When an explicit
[`--from <SRC>`](076_from.md) finds a non-empty copy of the same session
already in the target's storage (a prior clone, possibly since diverged), the
default is to overwrite it with a fresh copy of the source. `--keep-clone`
preserves the existing copy instead — only its mtime is refreshed so `-c`
continuation still selects it, and the source is not re-copied.

- **Type:** bool
- **Default:** false (existing non-empty destination copy is overwritten by a fresh clone)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"keep-clone"`

```sh
clr --from ../project-a "task"               # default: stale copy re-cloned from source
clr --from ../project-a --keep-clone "task"  # preserve the target's own diverged copy
CLR_KEEP_CLONE=1 clr --from ../project-a "task"  # env form of the same opt-out
```

**Rationale:** an explicit `--from` means "clone from there, now" — a stale copy
left behind by an earlier clone must not silently take precedence over the
freshly requested source. The pre-existing copy may nonetheless carry local
divergence worth keeping (turns added after the first clone); `--keep-clone`
is the deliberate way to say so.

**Both outcomes are announced on stderr** (suppressed by `--quiet`):

| Path | Message |
|------|---------|
| default (overwrite) | `[Runner] re-cloning over existing session copy <dest> (use --keep-clone to preserve it)` |
| `--keep-clone` (preserve) | `[Runner] kept existing session copy <dest> (--keep-clone; source not re-copied)` |

**Inert outside the collision case.** The flag only matters when a transplant is
planned (explicit cross-directory `--from`) AND the destination already holds a
non-empty file under the same session UUID. With no `--from`, an empty/missing
destination, fork-mode topics, `--new-session`, or a self-copy (source storage ==
target storage), there is nothing to keep or overwrite and the flag changes
nothing.

**Env var:** `CLR_KEEP_CLONE` — accepts `1` or `true` (case-insensitive); applied
when `--keep-clone` is absent from the CLI.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | see group listing |

### Referenced Parameters

| # | Parameter | Relationship |
|---|-----------|--------------|
| 076 | [`--from`](076_from.md) | The transplant this flag modifies — collision default lives in its step 7 |
| 074 | [`--quiet`](074_quiet.md) | Suppresses both announcement messages |
| 007 | [`--new-session`](007_new_session.md) | Suppresses the transplant entirely — `--keep-clone` inert |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |
| 11 | [`topic`](../command/11_topic.md) | false | Only reachable in dir-mode topics (fork mode plans no transplant) |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 28 | [028_session_transplant.md](../user_story/028_session_transplant.md) | Developer |
