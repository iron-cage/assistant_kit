# Command :: 3. `.show`

### Scope

- **Purpose**: Specify the `.show` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.show`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Display session or project details. Scope-aware: when `session_id::` is given without `project::`, the current project and all its topic variants (`--commit`, `--default-topic`, etc.) are searched (scope::local). Without `session_id::`, resolves to the current project. Use this when you need the content of a conversation or a project's session list.

**Parameters:** `session_id::`, `project::`, `show_entries::`, `show_metadata::`, `show_stat::`, `show_tokens::`, `scope::`, `path::`

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
| `show_stat::` | Boolean | optional | `0` | Append statistics footer in content mode |
| `show_tokens::` | Boolean | optional | `0` | Include token usage section |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Project search boundary (Case 2 only: session_id without project::) |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Scope anchor path |

`session_id::` and `project::` belong to [Session Identification](../param_group/03_session_identification.md) and [Project Scope](../param_group/02_project_scope.md) groups. `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md). `show_stat::` and `show_tokens::` belong to the [Output Control group](../param_group/01_output_control.md).

**Algorithm (4 steps):**
1. Parse and validate parameters — reject whitespace-only `session_id::`, reject `show_entries::` without `session_id::`
2. Dispatch by parameter combination — (a) no params → cwd project session list, (b) `session_id::` only → search cwd project and all topic variants, (c) `project::` only → that project session list, (d) both → that session in that project
3. Load project/session data — prefix matching for partial UUIDs (Git-style 8-char prefix)
4. Format output — metadata mode (`show_metadata::1`: structured fields) or content mode (default: conversation chat-log with separators); append optional stat footer (`show_stat::1`) and token usage (`show_tokens::1`)

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
- `show_stat::1` has no effect in `show_metadata::1` mode (metadata mode always shows structured fields)

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `show_tree::` |
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 3 | [Session Identification](../param_group/03_session_identification.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 3 | [`show_entries::`](../param/03_entries.md) | Boolean | optional |
| 6 | [`show_metadata::`](../param/06_metadata.md) | Boolean | optional |
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | optional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 14 | [`session_id::`](../param/14_session_id.md) | [`SessionId`](../type/09_session_id.md) | optional |
| 19 | [`show_stat::`](../param/19_show_stat.md) | Boolean | optional |
| 23 | [`show_tokens::`](../param/23_show_tokens.md) | Boolean | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
