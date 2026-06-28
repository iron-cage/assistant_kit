# Command :: 9. `.project.exists`

### Scope

- **Purpose**: Specify the `.project.exists` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.project.exists`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Check whether a directory has Claude Code conversation history. Exits with code `1` when no history is found, making it ideal for shell conditional logic.

**Parameters:** `path::`, `topic::`

**Exit:** `0` history found | `1` no history found

**Syntax:**
```bash
claude_storage .project.exists
claude_storage .project.exists path::PATH
claude_storage .project.exists topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Directory to check |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | — | Session topic suffix (without leading `-`) |

**Algorithm (2 steps):**
1. Resolve base path (explicit `path::` or cwd) and apply topic suffix if given
2. Check for conversation history via `check_continuation` — exit 0 with `"sessions exist"` or exit 1 with `"no sessions"`

**Output:**
- Exit 0: `"sessions exist\n"` on stdout
- Exit 1: `"no sessions"` on stderr

**Examples:**
```bash
# Check current directory
claude_storage .project.exists

# Check specific directory
claude_storage .project.exists path::/home/user/project

# Shell conditional
if clg .project.exists; then echo "Has history"; else echo "Fresh start"; fi
```

**Notes:**
- Exit code `1` is an informational result (no history found), not a command error
- This is the sole history-check command; `.session` was removed as a duplicate (task-014)

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
