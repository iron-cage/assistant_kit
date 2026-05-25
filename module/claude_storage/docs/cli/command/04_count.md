# Command :: 4. `.count`

Fast counting of projects, sessions, or entries without loading full content. Optimized for performance on large storage (2000+ projects). Use this when you need a number, not a listing.

**Parameters:** `target::`, `project::`, `session::`, `scope::`, `path::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .count
claude_storage .count target::sessions project::PROJECT
claude_storage .count target::entries project::PROJECT session::SESSION
claude_storage .count target::conversations project::PROJECT
claude_storage .count scope::relevant
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `target::` | [`TargetType`](../type/11_target_type.md) | optional | `projects` | What to count (`projects`, `sessions`, `entries`, `conversations`) |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | — | Scope to this project |
| `session::` | [`SessionId`](../type/09_session_id.md) | optional | — | Scope to this session |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `global` | Count boundary |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Scope anchor path |

`project::` belongs to the [Project Scope group](../param_group/02_project_scope.md). `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md).

**Examples:**
```bash
# Count all projects
claude_storage .count

# Count sessions in a specific project
claude_storage .count target::sessions project::abc123

# Count entries in a specific session
claude_storage .count target::entries project::abc123 session::xyz789

# Count conversations in a specific project
claude_storage .count target::conversations project::abc123

# Count sessions in the relevant scope (ancestor chain of cwd)
claude_storage .count target::sessions scope::relevant
```

**Notes:**
- `target::sessions` requires `project::` to avoid counting all sessions in all projects
- `target::entries` requires both `project::` and `session::`
- `target::conversations` requires `project::` (currently 1:1 with sessions; will differ once chain detection is implemented)

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Partial | `agent::`, `min_entries::` |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
