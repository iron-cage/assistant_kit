# Command :: 5. `.search`

### Scope

- **Purpose**: Specify the `.search` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.search`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Search session content for a query string across projects and sessions. Use this to find conversations by topic, code fragment, or any text that appeared in a session.

**Parameters:** `query::`, `project::`, `session::`, `case_sensitive::`, `entry_type::`, `scope::`, `path::`

**Exit:** `0` success | `1` argument error (missing `query::`) | `2` storage read error

**Syntax:**
```bash
claude_storage .search query::QUERY
claude_storage .search query::QUERY project::PROJECT
claude_storage .search query::QUERY [case_sensitive::1] [entry_type::user|assistant]
claude_storage .search query::QUERY scope::relevant
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query::` | String | **required** | — | Search query (alias: `q`) |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | — | Restrict to this project |
| `session::` | [`SessionId`](../type/09_session_id.md) | optional | — | Restrict to this session |
| `case_sensitive::` | Boolean | optional | `0` | Enable case-sensitive matching |
| `entry_type::` | [`EntryType`](../type/02_entry_type.md) | optional | `all` | Filter by entry type |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `global` | Search boundary |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Scope anchor path |

`project::` belongs to the [Project Scope group](../param_group/02_project_scope.md). `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md). `.search` has no output-control parameters.

**Algorithm (4 steps):**
1. Validate `query::` — reject missing or whitespace-only values; build search filter with case sensitivity and entry type
2. Determine search scope — specific session (prefix-matched), specific project, or all projects globally
3. Iterate sessions in scope — parse JSONL entries, apply filter (text match + optional entry type); skip corrupted sessions with warning
4. Format output — match count header; per-match: session ID, entry type, content excerpt

**Examples:**
```bash
# Search all storage for a term
claude_storage .search query::error

# Search with phrase and case sensitivity
claude_storage .search query::"session management" case_sensitive::1

# Search only user messages in a project
claude_storage .search query::implement entry_type::user project::abc123

# Search only within the relevant scope (ancestor chain of cwd)
claude_storage .search query::error scope::relevant
```

**Notes:**
- `query::` is required; command exits with `1` if omitted
- Use `q` alias for shorter syntax: `claude_storage .search q::version_bump`
- Without `project::`, searches all projects (may be slow on large storage); `scope::` is a more precise alternative for limiting the search boundary

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Partial | `agent::`, `min_entries::` |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 2 | [`case_sensitive::`](../param/02_case_sensitive.md) | Boolean | optional |
| 4 | [`entry_type::`](../param/04_entry_type.md) | [`EntryType`](../type/02_entry_type.md) | optional |
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | optional |
| 11 | [`query::`](../param/11_query.md) | String | **required** |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 13 | [`session::`](../param/13_session.md) | [`SessionId`](../type/09_session_id.md) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
