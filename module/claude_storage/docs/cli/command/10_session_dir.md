# Command :: 10. `.session.dir`

### Scope

- **Purpose**: Specify the `.session.dir` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.session.dir`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Compute the session working directory path (`{base}/-{topic}`) without creating it. Use this to determine the correct session directory before deciding whether to start or resume.

**Parameters:** `path::`, `topic::`

**Exit:** `0` success | `1` argument error (invalid path or topic)

**Syntax:**
```bash
claude_storage .session.dir
claude_storage .session.dir path::PATH
claude_storage .session.dir path::PATH topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Base directory |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | `default_topic` | Session topic (without leading `-`) |

**Algorithm (2 steps):**
1. Resolve base path (explicit `path::` or cwd) and topic (explicit or `default_topic`); validate topic (non-empty, no path separators)
2. Compute and output session directory path `{base}/-{topic}`

**Output:** Single line — the absolute path to `{base}/-{topic}`.

**Examples:**
```bash
# Session dir for current directory with default topic
claude_storage .session.dir

# Session dir for specific project
claude_storage .session.dir path::/home/user/project

# Session dir with custom topic
claude_storage .session.dir path::/home/user/project topic::work
```

**Notes:**
- `path::` defaults to cwd when omitted
- The returned directory path does not need to exist on disk
- Use `.session.ensure` to create the directory and detect resume strategy

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
