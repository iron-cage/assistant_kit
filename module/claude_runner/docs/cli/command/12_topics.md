# CLI Command: topics

### Description

Read-only counterpart to [`topic`](11_topic.md): list the topics under a base (both mechanisms — fork-mode sessions and legacy topic directories), or resolve one topic name to the absolute path `topic` would use for it in either mechanism. Runs nothing, spawns no subprocess, and creates no directories.

-- **Parameters:** `--path`, `--file`, `--dir`/`--to`, `-g`/`--global`
-- **Exit Codes:** 0 (success, including "no topics found") | 1 (error)

### Syntax

```sh
clr topics [--dir <PATH>] [--global]
clr topics --path <NAME> [--dir <PATH>] [--global]
clr topics --file <NAME> [--dir <PATH>] [--global]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `--path <NAME>` | string | — | Resolve a topic name to its DIR-mode directory (`<base>/-<NAME>`) and exit; suppresses listing |
| `--file <NAME>` | string | — | Resolve a topic name to its FORK-mode session file (`<storage of base>/<UUIDv5>.jsonl`) and exit; suppresses listing |
| [`--dir <PATH>`](../param/008_dir.md) | path | CWD | Base directory to look for topics under |
| [`-g`/`--global`](../param/087_global.md) | flag | off | Use the global topic home as the base instead of CWD |
| `-h`/`--help` | — | — | Print `topics` subcommand help and exit 0 |

`--path` and `--file` are mutually exclusive — one name, two disjoint resolution rules (dir-mode directory vs fork-mode session file); asking for both at once is a contradiction and exits 1.

**Base resolution** is identical to `--topic`'s, computed by the same `claude_topic_core::identity::topic_base()`: `--dir` if given, else the global topic home if `--global`, else CWD. An explicit `--dir` outranks `--global`.

**Algorithm — list form (4 steps):**
1. Resolve the base directory.
2. Dir-mode topics: read the base's direct entries, keeping directories whose name starts with `-`; the topic name is that name with the leading `-` stripped (a bare `-` is not a topic). Sessions = count of `*.jsonl` files in that directory's own Claude Code session storage (via `scope_for()`).
3. Fork-mode topics: read the names recorded for the canonical base in the topics registry (`CLR_TOPIC_REGISTRY_DIR` > `~/.clr/topics/`); path = the shared UUIDv5 session file. Sessions = 1 when that file exists non-empty, 0 otherwise (a registry entry whose file was deleted stays listed — its name is still reserved for auto-naming).
4. Merge and print the rows sorted by name, then mode. The same name can legitimately exist once per mode — both rows are shown.

**Algorithm — resolve forms (2 steps each):**
1. Resolve the base directory.
2. `--path`: print `<base>/-<NAME>` and exit 0 — the filesystem is never consulted. `--file`: print `<storage of canonical base>/<UUIDv5( canonical base, NAME )>.jsonl` and exit 0 — the file need not exist; only the storage-root resolution reads the environment (HOME). Both answers are the same whether or not the topic exists yet.

### Output Format

List form:

```sh
NAME              MODE  SESSIONS  PATH
auth-refactor     dir          3  /home/alice/project/-auth-refactor
flaky-test        fork         1  /home/alice/.claude/projects/-home-alice-project/41299c24-a8f5-589f-9fce-8474fc855532.jsonl
```

`NAME` is left-aligned and padded to the longest name (minimum width 4); `MODE` is `dir` or `fork`, left-aligned in a 4-column field; `SESSIONS` is right-aligned in an 8-column field.

When the base holds no topic directories, the listing goes to **stderr** and the exit code is still 0 — an empty result is not an error:

```sh
no topics in /home/alice/project
```

Resolve forms — one absolute path on stdout, nothing else:

```sh
$ clr topics --path auth-refactor
/home/alice/project/-auth-refactor
$ clr topics --file flaky-test
/home/alice/.claude/projects/-home-alice-project/41299c24-a8f5-589f-9fce-8474fc855532.jsonl
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Topics listed, no topics found, or a path resolved |
| 1 | Error: unknown option, `--path`/`--file`/`--dir` missing its value, `--path`/`--file` value containing `/`, `--file` value empty, `--path` combined with `--file`, or (`--file` only) unresolvable session storage (HOME unset) |

### Examples

```sh
# List topics in the current project
clr topics

# List topics in the global topic home
clr topics --global

# List topics under an explicit base
clr topics --dir /home/alice/project

# Where would (or does) this dir-mode topic's directory live?
clr topics --path auth-refactor
clr topics --global --path auth-refactor

# Where is this fork-mode topic's session file?
clr topics --file flaky-test

# Inspect a fork topic's usage directly from its session file
jq '.message.usage' "$( clr topics --file flaky-test )"

# Byte-identical cross-check from the storage CLI
claude_storage .session.path path::. topic::flaky-test

# Recover a global topic's path in a later shell, by name alone
cd "$( clr topics --global --path auth-refactor )"

# Enumerate for scripting — NAME is the first column
clr topics --global | tail -n +2 | awk '{ print $1 }'
```

### Notes

**Deterministic by construction.** `topics --path NAME` and the directory `topic`/`--topic NAME` actually runs in are computed by the same `claude_topic_core::topic_dir()` call; `topics --file NAME` and the session id a fork-mode run uses are computed by the same `claude_storage_core::topic_session_file()`/`topic_session_id()` rule — so neither resolve form can ever disagree with a real run. That is what makes a global topic addressable from its name alone, and a fork topic's session file scriptable, in any later shell.

**`--file` is byte-identical to `claude_storage .session.path`.** Both delegate to `claude_storage_core::topic_session_file` keyed on the canonical physical base: `clr topics --file NAME` ≡ `claude_storage .session.path path::<base> topic::NAME`. Verify: run both against the same base and `diff` the outputs.

**`--path`/`--file` never require the topic to exist.** They answer "where would this topic live?", not "does it exist?". Use the list form (or `test -d` for a directory, `test -s` for a session file) when existence matters.

**Session count semantics differ by mode.** Dir rows count `*.jsonl` files in the topic directory's own storage (0 for a never-entered topic — `resolve_effective_dir()` creates the directory, but its storage is created by Claude Code itself on first run). Fork rows are 1 when the topic's session file exists non-empty, else 0 — a fork topic IS a single session, so the column reads as an existence flag there. A topic created by a `--dry-run` invocation has no side effects and so never appears in the listing (fork registry recording is a run-path effect too).

**Fork listing depends on the registry.** UUIDv5 is one-way — the name cannot be recovered from a session file — so fork rows come from the topics registry (`CLR_TOPIC_REGISTRY_DIR` > `~/.clr/topics/`, one file per base, one name per line, append-if-missing, warn-never-fatal). The registry is a convenience index, never an authority: the authoritative existence signal for a fork topic is its session file. A fork topic created on a machine (or under an env) whose registry entry was lost still works by name — it just does not appear in the listing.

**`--path`/`--file` take a single name component.** Values containing `/` are rejected with exit 1, mirroring `--topic`'s own BUG-230 guard — a topic name is a name, never a path. `--file` additionally rejects an empty name (an empty UUIDv5 input would still resolve, to a meaningless session); `--path ""` keeps its historical behavior of printing `<base>/-`.

### Referenced Command Group

Evaluated against every existing command under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify; `topics` opens Singleton Group 10. `dispatch_topics()` (`src/cli/topics.rs:66`) has zero cross-calls with any other dispatch function: it never calls `dispatch_run()`, and `dispatch_topic()` never calls it. Against the nearest candidate, [`topic`](11_topic.md): `topic` executes a Claude subprocess and accepts `run`'s full ~40-parameter surface, while `topics` executes nothing and accepts 4 parameters — no default value of any `topic` parameter yields a directory listing, so the Representation Absorption Test fails on both the handler and the parameter set. The shared `claude_topic_core` helpers are an internal path-computation module, not a shared dispatch function — the same distinction that keeps [`scope`](09_scope.md) out of `run`'s group over `scope_for()`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 11 | [`topic`](11_topic.md) | Write counterpart — creates/enters what `topics` reports on; both resolve paths through `claude_topic_core` |
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
