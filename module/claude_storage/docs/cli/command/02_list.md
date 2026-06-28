# Command :: 2. `.list`

### Scope

- **Purpose**: Specify the `.list` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.list`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

List projects or conversations in Claude Code storage. Project-first view: all projects are listed, with conversations optionally shown per project. Use this when navigating projects or filtering by project path.

**Parameters:** `type::`, `path::`, `show_sessions::`, `session::`, `agent::`, `min_entries::`, `scope::`, `project::`, `count::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .list
claude_storage .list type::uuid
claude_storage .list path::SUBSTR [show_sessions::1]
claude_storage .list session::FILTER [agent::0|1] [min_entries::N]
claude_storage .list scope::relevant
claude_storage .list type::conversation project::PROJECT
claude_storage .list type::conversation count::1 project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `type::` | [`ProjectType`](../type/06_project_type.md) | optional | `all` | Project naming filter (`uuid`, `path`, `all`, `conversation`) |
| `path::` | [`PathSubstring`](../type/04_path_substring.md) | optional | — | Filter projects by path substring |
| `show_sessions::` | Boolean | optional | `0` | Show sessions per project |
| `session::` | [`SessionFilter`](../type/08_session_filter.md) | optional | — | Filter sessions by ID substring |
| `agent::` | Boolean | optional | — | Session type filter (`0`=main, `1`=agent) |
| `min_entries::` | [`EntryCount`](../type/01_entry_count.md) | optional | — | Minimum entry count threshold |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `global` | Project discovery boundary |
| `project::` | String | required for `type::conversation` | — | Project ID; scopes conversation listing |
| `count::` | Boolean | optional | `0` | Output only the count as a bare integer |

Session filter parameters belong to the [Session Filter group](../param_group/04_session_filter.md). See [Scope Configuration group](../param_group/05_scope_configuration.md) for `scope::` semantics.

**Algorithm (5 steps):**
1. Early dispatch — if `type::conversation`: load project, build session families, group into conversations, output conversation IDs (or bare count when `count::1`)
2. Parse filter parameters — resolve `path::` (smart shell semantics: `.`, `..`, `~`), validate `min_entries::`, detect session filter presence
3. Auto-enable session display — any session filter (`session::`, `agent::`, `min_entries::`) implicitly sets `show_sessions::1`
4. List and filter projects by type (`uuid`/`path`/`all`) and path substring
5. Format output — project IDs with conversation count; if sessions enabled, list filtered sessions per project

**Examples:**
```bash
# List all projects
claude_storage .list

# List all sessions for projects matching path
claude_storage .list path::assistant show_sessions::1

# Find sessions matching a topic filter
claude_storage .list session::commit

# Find agent sessions with at least 10 entries
claude_storage .list agent::1 min_entries::10

# List only projects in the ancestor chain of cwd
claude_storage .list scope::relevant

# List conversation IDs for a specific project
claude_storage .list type::conversation project::abc123

# Count conversations in a project (bare integer output)
claude_storage .list type::conversation count::1 project::abc123
```

**Notes:**
- `session::`, `agent::`, or `min_entries::` auto-enables `show_sessions::1`; use `show_sessions::0` to suppress
- `type::uuid` shows projects identified by UUID rather than path encoding
- `type::conversation` requires `project::` and lists one conversation ID per line
- `count::1` with `type::conversation` outputs only the count as a bare integer (useful for scripting)
- `scope::global` is the default — lists all projects regardless of cwd; `scope::relevant` lists only projects in the ancestor chain of cwd

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `path::` (PathSubstring) |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 1 | [`agent::`](../param/01_agent.md) | Boolean | optional |
| 7 | [`min_entries::`](../param/07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | optional |
| 9 | [`path::`](../param/09_path.md) | [`PathSubstring`](../type/04_path_substring.md) | optional |
| 10 | [`project::`](../param/10_project.md) | String | conditional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 13 | [`session::`](../param/13_session.md) | [`SessionFilter`](../type/08_session_filter.md) | optional |
| 15 | [`show_sessions::`](../param/15_sessions.md) | Boolean | optional |
| 18 | [`type::`](../param/18_type.md) | [`ProjectType`](../type/06_project_type.md) | optional |
| 21 | [`count::`](../param/21_count.md) | Boolean | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
