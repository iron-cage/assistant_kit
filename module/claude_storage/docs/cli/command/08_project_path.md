# Command :: 8. `.project.path`

### Scope

- **Purpose**: Specify the `.project.path` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.project.path`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Compute the Claude Code storage path for a directory without requiring it to exist. Use this to inspect what storage path would be used for a given working directory.

**Parameters:** `path::`, `topic::`

**Exit:** `0` success | `1` argument error (invalid path or topic)

**Syntax:**
```bash
claude_storage .project.path
claude_storage .project.path path::PATH
claude_storage .project.path topic::TOPIC
claude_storage .project.path path::PATH topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Directory to compute storage path for |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | — | Session topic suffix (without leading `-`) |

**Algorithm (2 steps):**
1. Resolve base path (explicit `path::` or cwd) and apply topic suffix if given (`-{topic}`)
2. Compute storage path via `encode_path` and output `~/.claude/projects/{encoded}/`

**Output:** Single line — the absolute path to `~/.claude/projects/{encoded}/` (or `{encoded}-{topic}/` when `topic::` given).

**Examples:**
```bash
# Storage path for current directory
claude_storage .project.path

# Storage path for a specific directory
claude_storage .project.path path::/home/user/project

# Storage path with topic suffix
claude_storage .project.path topic::default_topic

# Storage path with directory and topic
claude_storage .project.path path::~/projects/myapp topic::work
```

**Notes:**
- The returned path does not need to exist on disk
- Use `.project.exists` to test whether the path has conversation history

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 17 | [`topic::`](../param/17_topic.md) | [`TopicName`](../type/12_topic_name.md) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
