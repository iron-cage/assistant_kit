# CLI Command: topics

### Description

Read-only counterpart to [`topic`](11_topic.md): list the topic directories under a base, or resolve one topic name to the absolute path `topic` would use for it. Runs nothing, spawns no subprocess, and creates no directories.

-- **Parameters:** `--path`, `--dir`/`--to`, `-g`/`--global`
-- **Exit Codes:** 0 (success, including "no topics found") | 1 (error)

### Syntax

```sh
clr topics [--dir <PATH>] [--global]
clr topics --path <NAME> [--dir <PATH>] [--global]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `--path <NAME>` | string | — | Resolve a single topic name to its absolute path and exit; suppresses listing |
| [`--dir <PATH>`](../param/008_dir.md) | path | CWD | Base directory to look for topic directories under |
| [`-g`/`--global`](../param/087_global.md) | flag | off | Use the global topic home as the base instead of CWD |
| `-h`/`--help` | — | — | Print `topics` subcommand help and exit 0 |

**Base resolution** is identical to `--topic`'s, computed by the same `src/cli/topic_path.rs::topic_base()`: `--dir` if given, else the global topic home if `--global`, else CWD. An explicit `--dir` outranks `--global`.

**Algorithm — list form (3 steps):**
1. Resolve the base directory.
2. Read the base's direct entries, keeping directories whose name starts with `-`; the topic name is that name with the leading `-` stripped (a bare `-` is not a topic).
3. For each, count `*.jsonl` files in that directory's own Claude Code session storage (via `scope_for()`), then print the rows sorted by name.

**Algorithm — resolve form (2 steps):**
1. Resolve the base directory.
2. Print `<base>/-<NAME>` and exit 0. This is a pure computation — the filesystem is never consulted, so the answer is the same whether or not the topic exists yet.

### Output Format

List form:

```sh
NAME              SESSIONS  PATH
auth-refactor            3  /home/alice/project/-auth-refactor
flaky-test               1  /home/alice/project/-flaky-test
```

`NAME` is left-aligned and padded to the longest name (minimum width 4); `SESSIONS` is right-aligned in an 8-column field.

When the base holds no topic directories, the listing goes to **stderr** and the exit code is still 0 — an empty result is not an error:

```sh
no topics in /home/alice/project
```

Resolve form — one absolute path on stdout, nothing else:

```sh
$ clr topics --path auth-refactor
/home/alice/project/-auth-refactor
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Topics listed, no topics found, or a path resolved |
| 1 | Error: unknown option, `--path`/`--dir` missing its value, or `--path` value containing `/` |

### Examples

```sh
# List topics in the current project
clr topics

# List topics in the global topic home
clr topics --global

# List topics under an explicit base
clr topics --dir /home/alice/project

# Where would (or does) this topic live?
clr topics --path auth-refactor
clr topics --global --path auth-refactor

# Recover a global topic's path in a later shell, by name alone
cd "$( clr topics --global --path auth-refactor )"

# Enumerate for scripting — NAME is the first column
clr topics --global | tail -n +2 | awk '{ print $1 }'
```

### Notes

**Deterministic by construction.** `topics --path NAME` and the directory `topic`/`--topic NAME` actually runs in are computed by the same `topic_path::topic_dir()` call, so the two can never disagree. That is what makes a global topic addressable from its name alone, from any working directory, in any later shell.

**`--path` never touches the disk.** It answers "where would this topic live?", not "does it exist?". Use the list form (or a plain `test -d`) when existence matters.

**Session count is 0 for a never-entered topic.** `resolve_effective_dir()` creates the topic directory before spawn, but the Claude Code session storage under it is created by Claude Code itself on first run. A topic created by a `--dry-run` invocation is not created at all (dry-run is side-effect-free) and so never appears in the listing.

**`--path` takes a single name component.** Values containing `/` are rejected with exit 1, mirroring `--topic`'s own BUG-230 guard — a topic name is a directory name, never a path.

### Referenced Command Group

Evaluated against every existing command under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify; `topics` opens Singleton Group 10. `dispatch_topics()` (`src/cli/topics.rs:66`) has zero cross-calls with any other dispatch function: it never calls `dispatch_run()`, and `dispatch_topic()` never calls it. Against the nearest candidate, [`topic`](11_topic.md): `topic` executes a Claude subprocess and accepts `run`'s full ~40-parameter surface, while `topics` executes nothing and accepts 3 parameters — no default value of any `topic` parameter yields a directory listing, so the Representation Absorption Test fails on both the handler and the parameter set. The shared `topic_path` helpers are an internal path-computation module, not a shared dispatch function — the same distinction that keeps [`scope`](09_scope.md) out of `run`'s group over `scope_for()`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 11 | [`topic`](11_topic.md) | Write counterpart — creates/enters what `topics` reports on; both resolve paths through `topic_path` |
| 9 | [`scope`](09_scope.md) | Same shape — read-only, `--dir`-based path inspection that runs no subprocess |
| 6 | [`ps`](06_ps.md) | Also an aligned-column listing of session state, keyed by running process rather than by topic directory |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--dir` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 22 | [022_session_isolation_topic.md](../user_story/022_session_isolation_topic.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |
| 31 | [031_topic_discovery.md](../user_story/031_topic_discovery.md) | Developer |

---

**Category:** Inspection / path resolution
**Complexity:** 5
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Zero (read-only)
