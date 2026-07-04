# Command :: 12. `.tail`

### Scope

- **Purpose**: Specify the `.tail` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.tail`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Print the last N entries of the current directory's conversation. Resolves cwd to its project and the `default_topic` session by default, then prints the last 4 entries — no parameters required. Use this for a quick content refresher without running a lookup command first.

**Parameters:** `tail::`, `path::`, `topic::`

**Exit:** `0` success | `1` argument error | `2` storage read error or project not found

**Syntax:**
```bash
claude_storage .tail
claude_storage .tail tail::N
claude_storage .tail topic::TOPIC
claude_storage .tail path::PATH tail::N topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `tail::` | Integer | optional | `4` | Number of trailing entries to print; `0` shows all entries |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Directory to resolve the project from |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | `default_topic` | Session topic suffix to resolve |

**Algorithm (5 steps):**
1. Resolve `path::` (default cwd) to a project ID
2. Resolve the session for `topic::` (default `default_topic`) within that project
3. Load session entries; exit `2` if the project or session is not found
4. Take the last `tail::` entries (default `4`); `tail::0` takes all entries; fewer available entries than requested yields all available
5. Format and print entries as conversation chat-log content, oldest-first

**Examples:**
```bash
# Print the last 4 entries for the current directory (default)
claude_storage .tail

# Print the last 10 entries
claude_storage .tail tail::10

# Print all entries, oldest-first
claude_storage .tail tail::0

# Print the last 4 entries of a non-default topic
claude_storage .tail topic::work

# Resolve a different directory
claude_storage .tail path::/home/alice/projects/my-app tail::6
```

**Notes:**
- Zero-parameter invocation always works: cwd → project → `default_topic` session → last 4 entries
- `tail::0` prints all entries, oldest-first — the full-history equivalent within the resolved session
- Exits `2` when the resolved project or session has no history, matching `.show`'s not-found convention
- Deliberately minimal parameter surface — does not expose `session_id::`, `project::`, or display-mode toggles; use `.show` for full inspection

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 17 | [`topic::`](../param/17_topic.md) | [`TopicName`](../type/12_topic_name.md) | optional |
| 25 | [`tail::`](../param/25_tail.md) | Integer | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |
