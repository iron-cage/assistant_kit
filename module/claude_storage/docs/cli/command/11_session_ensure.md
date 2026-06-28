# Command :: 11. `.session.ensure`

### Scope

- **Purpose**: Specify the `.session.ensure` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.session.ensure`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Ensure a session working directory exists, creating it if necessary. Reports whether the session has existing conversation history (`resume`) or is starting fresh (`fresh`). Outputs two lines: the absolute session directory path and the strategy.

**Parameters:** `path::`, `topic::`, `strategy::`

**Exit:** `0` success | `1` argument error (invalid path or params)

**Syntax:**
```bash
claude_storage .session.ensure
claude_storage .session.ensure path::PATH
claude_storage .session.ensure path::PATH topic::TOPIC
claude_storage .session.ensure path::PATH strategy::resume|fresh
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Base directory |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | `default_topic` | Session topic (without leading `-`) |
| `strategy::` | [`StrategyType`](../type/13_strategy_type.md) | optional | auto-detect | Override resume strategy |

**Algorithm (3 steps):**
1. Resolve session directory path — base from `path::` (or cwd), topic from `topic::` (or `default_topic`); validate topic and optional `strategy::` value
2. Create directory if absent — idempotent `create_dir_all` (never wipes existing content)
3. Detect strategy — `resume` if storage history exists for this session dir, `fresh` otherwise; `strategy::` overrides the detected label without modifying filesystem

**Output** (two lines):
```
{absolute session dir path}
{resume|fresh}
```

**Strategy detection** (when `strategy::` not provided):
- `resume` — storage history exists for the session directory
- `fresh` — no conversation history found

**Examples:**
```bash
# Ensure session dir with default topic (auto-detect strategy)
claude_storage .session.ensure path::/home/user/project

# With custom topic
claude_storage .session.ensure path::/home/user/project topic::work

# Force strategy
claude_storage .session.ensure path::/home/user/project strategy::resume
```

**Notes:**
- Creates `{base}/-{topic}` directory if it does not exist
- `path::` defaults to cwd when omitted
- When `strategy::resume` is forced but no history exists, the output still reports `resume` (caller's intent is respected)

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 17 | [`topic::`](../param/17_topic.md) | [`TopicName`](../type/12_topic_name.md) | optional |
| 20 | [`strategy::`](../param/20_strategy.md) | [`StrategyType`](../type/13_strategy_type.md) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
