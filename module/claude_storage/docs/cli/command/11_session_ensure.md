# Command :: 11. `.session.ensure`

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
| `topic::` | [`TopicName`](../type/13_topic_name.md) | optional | `default_topic` | Session topic (without leading `-`) |
| `strategy::` | [`StrategyType`](../type/14_strategy_type.md) | optional | auto-detect | Override resume strategy |

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

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
