# User Story 031: Topic Discovery

### Scope

- **Persona**: Developer
- **Goal**: Find out which topic sessions exist and where a given topic name lives, without running Claude Code and without remembering which directory the topic was created from.

### User Story

> As a developer,
> I want to list my topic sessions and resolve a topic name to its absolute path,
> so I can return to a topic days later from any shell, and inspect or clean up topics I have accumulated.

### Acceptance Criteria

- **AC-1 (List local):** `clr topics` lists every `-<name>` directory directly under CWD, one row per topic, sorted by name.
- **AC-2 (List global):** `clr topics --global` lists the topics under the global topic home instead of CWD.
- **AC-3 (Explicit base):** `clr topics --dir /home/alice/project` lists topics under the given base; `--dir` outranks `--global` when both are given.
- **AC-4 (Session count):** each row reports how many `*.jsonl` session files exist in that topic's own Claude Code storage; a topic that has never been entered reports `0`.
- **AC-5 (Empty base):** a base with no topics prints `no topics in <base>` to **stderr** and exits 0 — an empty result is not an error.
- **AC-6 (Resolve by name):** `clr topics --path <NAME>` prints the single absolute path `--topic <NAME>` would resolve to and exits 0.
- **AC-7 (Resolve is pure):** the resolved path is identical whether or not the topic exists on disk, and resolving never creates anything.
- **AC-8 (Determinism):** the path from `clr topics --path NAME` is byte-identical to the effective directory shown by `clr --dry-run --topic NAME` under the same base flags.
- **AC-9 (Name validation):** a `--path` value containing `/` is rejected with exit 1, matching `--topic`'s own single-name-component constraint.
- **AC-10 (Read-only):** no Claude subprocess is spawned and no directory is created by any form of the command.

### Primary Flags

| Flag | Role |
|------|------|
| (none) | List topics under CWD |
| `--global` / `-g` | Use the global topic home as the base |
| `--dir <PATH>` | Use an explicit base; outranks `--global` |
| `--path <NAME>` | Resolve one name to its path instead of listing |

### Examples

```sh
# What topics do I have here?
clr topics

# What global topics do I have?
clr topics --global

# Where does this topic live?
clr topics --global --path auth-refactor
# /tmp/clr-topic/-auth-refactor

# Return to it from any shell
cd "$( clr topics --global --path auth-refactor )"

# Confirm the resolver agrees with the runner (AC-8)
clr topics --global --path auth-refactor
clr --dry-run --global --topic auth-refactor "x" | grep -o '/[^ ]*-auth-refactor'

# Continue a topic found by listing
clr topic --global --topic "$( clr topics --global | tail -n +2 | awk 'NR==1 { print $1 }' )" "Where were we?"
```

### Related Commands

| Command | Role |
|---------|------|
| `topics` | Primary command for this user story |
| `topic` | Creates and enters what `topics` reports on |
| `run` / `ask` | Accept the same `--topic` / `--global` base resolution |
| `scope` | Sibling read-only path inspector, keyed by directory rather than topic name |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../command/12_topics.md`](../command/12_topics.md) | `clr topics` command reference |
| [`../command/11_topic.md`](../command/11_topic.md) | `clr topic` command reference |
| [`../param/087_global.md`](../param/087_global.md) | `--global` base redirection |
| [`../param/028_topic.md`](../param/028_topic.md) | `--topic` naming and identity values |
| [`../../guide/001_topic_sessions.md`](../../guide/001_topic_sessions.md) | End-to-end walkthrough of the five topic scenarios |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 030 | [Topic Creation](030_topic_creation.md) | Creates the topics this story discovers |
| 022 | [Session Isolation via Topic Directory](022_session_isolation_topic.md) | The `--topic` isolation mechanism being enumerated |
| 029 | [Scope Inspection](029_scope_inspection.md) | Same read-only path-inspection shape, keyed by directory |
