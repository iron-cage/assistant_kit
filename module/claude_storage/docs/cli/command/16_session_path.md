# Command :: 16. `.session.path`

### Scope

- **Purpose**: Specify the `.session.path` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.session.path`.
- **In Scope**: Invocation syntax, accepted parameters, selector semantics, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`), the fork-topic UUIDv5 rule itself (→ `claude_storage_core::topic_session_id`).

**Representation Absorption Test** (per [`../command_group/readme.md`](../command_group/readme.md), the mandatory gate before adding any new command name): closest candidate is [`.session.dir`](10_session_dir.md) — nearest by name and by shared `path::`/`topic::` parameter names. Fails both criteria: (1) *identical routine* — `session_path_routine()` and `session_dir_routine()` are distinct functions with no cross-calls (the standard sweep for `session_path_routine` matches only the `src/cli_main.rs` phf registration and the `src/cli/mod.rs` re-export); `.session.dir` computes the session *working directory* `{base}/-{topic}` without ever touching Claude storage, while `.session.path` computes a *session transcript file* inside `~/.claude/projects/…` storage — including a `latest` selector that reads the disk to pick the most recent `.jsonl`, a code path `.session.dir` has no equivalent of. Not reachable by changing `.session.dir`'s parameter defaults. (2) *identical parameter set* — `.session.path` registers `session::` and `latest::`, neither of which `.session.dir` accepts; and although both accept a parameter spelled `topic::`, the semantics are disjoint (dir-suffix sense vs fork-mode UUIDv5 sense — see Topic Sense Collision below), so even the shared name is not a shared parameter. Second-closest candidate [`.project.path`](08_project_path.md) fails the same way: it returns the storage *directory* for a base dir; `.session.path` is that directory plus per-session file selection through three mutually exclusive selectors. Confirmed as a genuinely new command.

Resolve a session's absolute transcript file path (`…/.claude/projects/{encoded-base}/{session}.jsonl`) for a base directory. Use this to feed a session file directly to `jq`, `tail`, or any tool that reads the JSONL transcript — without hand-assembling the encoded storage path.

**Parameters:** `path::`, `session::`, `latest::`, `topic::`

**Exit:** `0` success | `1` argument error (mutually exclusive selectors; empty or slash-containing `session::`/`topic::`; storage resolution failure) | `2` no sessions (default/`latest::` selector on a storage with no qualifying session files)

**Syntax:**
```bash
claude_storage .session.path
claude_storage .session.path path::PATH
claude_storage .session.path path::PATH latest::1
claude_storage .session.path path::PATH session::UUID
claude_storage .session.path path::PATH topic::NAME
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Base directory whose storage is resolved |
| `session::` | [`SessionId`](../type/09_session_id.md) | optional | unset | Exact session UUID — pure join, no existence check |
| `latest::` | Boolean | optional | effective default | Most recently modified qualifying session in the storage |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | unset | Fork-mode topic name — resolved via the UUIDv5 rule (see Topic Sense Collision) |

`session::`, `latest::`, and `topic::` are **mutually exclusive selectors** — passing more than one is an argument error (exit 1). Omitting all three selects `latest` behavior.

**Algorithm (3 steps):**
1. Reject if more than one selector (`session::`, `latest::`, `topic::`) is present
2. Resolve the base (`path::` or cwd), canonicalize it physically, and map it to its Claude storage directory (`CLAUDE_HOME`, else `$HOME/.claude`, + `/projects/{encoded-base}`)
3. Select the file: `session::` → `{storage}/{id}.jsonl` unconditionally; `topic::` → `{storage}/{UUIDv5(canonical base, name)}.jsonl` unconditionally; default/`latest::` → most recently modified non-`agent-` non-empty `.jsonl` in the storage, or exit `2` with `no sessions in {storage}` on stderr when none qualify

**Output:** Single line — the absolute session file path. Only the `latest` selector reads the disk; `session::` and `topic::` are pure computations (the printed file need not exist).

**Topic Sense Collision (deliberate):** every other `claude_storage` command's `topic::` means the legacy dir-suffix sense — the value names a `-{topic}` *sibling directory* of the base (`{base}/-{topic}`), and paths are computed for that directory. THIS command's `topic::` means the fork-mode sense — the value names a deterministic session *inside the base directory's own storage*, whose filename is `UUIDv5(canonical physical base + NUL + topic name)`. The two senses share a parameter name because they answer the same user question ("where is topic X?") for the two topic mechanisms; the output is byte-identical to `clr topics --file NAME`, both delegating to `claude_storage_core::topic_session_file`.

**Examples:**
```bash
# Latest session file for the current directory (default selector)
claude_storage .session.path

# Explicit latest
claude_storage .session.path path::/home/user/project latest::1

# Exact session — pure join, works even before the file exists
claude_storage .session.path path::/home/user/project session::0a1b2c3d-...

# Fork-mode topic session file — byte-identical to `clr topics --file review`
claude_storage .session.path path::/home/user/project topic::review

# Feed the latest transcript to jq
jq '.message.usage' "$( claude_storage .session.path )"
```

**Notes:**
- `path::` defaults to cwd when omitted; the base is canonicalized (symlinks resolved) before encoding, so a symlinked path resolves to the same storage as its physical target
- Exit `2` (not `1`) for "no sessions" lets scripts distinguish an empty storage from a usage error
- `agent-*.jsonl` files and zero-length files never qualify for `latest`
- Use [`.session.dir`](10_session_dir.md) for the legacy dir-mode topic *working directory*; use `.session.path` for the transcript *file*

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 13 | [`session::`](../param/13_session.md) | [`SessionId`](../type/09_session_id.md) | optional |
| 41 | [`latest::`](../param/41_latest.md) | Boolean | optional |
| 17 | [`topic::`](../param/17_topic.md) | [`TopicName`](../type/12_topic_name.md) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
