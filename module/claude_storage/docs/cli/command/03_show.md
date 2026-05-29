# Command :: 3. `.show`

Display session or project details. Scope-aware: when `session_id::` is given without `project::`, the current project and all its topic variants (`--commit`, `--default-topic`, etc.) are searched (scope::local). Without `session_id::`, resolves to the current project. Use this when you need the content of a conversation or a project's session list.

**Parameters:** `session_id::`, `project::`, `verbosity::`, `show_entries::`, `show_metadata::`, `scope::`, `path::`

**Exit:** `0` success | `1` argument error | `2` storage read error or project not found

**Syntax:**
```bash
claude_storage .show
claude_storage .show session_id::ID
claude_storage .show project::PROJECT
claude_storage .show session_id::ID [show_entries::1] [show_metadata::1]
claude_storage .show session_id::ID project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id::` | [`SessionId`](../type/09_session_id.md) | optional | — | Session to display; when given without `project::`, the current project and all its topic variants are searched (scope::local) |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | current dir | Project identifier; when given with `session_id::`, restricts search to this project only |
| `show_entries::` | Boolean | optional | `0` | Show all entries in session |
| `show_metadata::` | Boolean | optional | `0` | Show metadata only (suppresses content) |
| `verbosity::` | [`VerbosityLevel`](../type/12_verbosity_level.md) | optional | `1` | Output detail level |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Project search boundary (Case 2 only: session_id without project::) |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Scope anchor path |

`session_id::` and `project::` belong to [Session Identification](../param_group/03_session_identification.md) and [Project Scope](../param_group/02_project_scope.md) groups. `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md).

**Examples:**
```bash
# Show current project's session list
claude_storage .show

# Show a specific session — searches all projects globally (no project:: needed)
claude_storage .show session_id::-default_topic

# Show session metadata only (no content)
claude_storage .show session_id::abc123 show_metadata::1

# Show a session in a specific project only (skips global search)
claude_storage .show session_id::ID project::/path/to/project
```

**Notes:**
- When `session_id::` is given without `project::`, the current project and all its topic variants (scope::local) are searched; supply `project::` to restrict lookup to one specific project
- Without `session_id::`, resolves to current directory project; exits with `2` if cwd has no project in storage
- `show_entries::1` and `show_metadata::1` are mutually exclusive; `show_entries::1` takes precedence

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | — |
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 3 | [Session Identification](../param_group/03_session_identification.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
