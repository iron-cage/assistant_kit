# Command :: 3. `.show`

### Scope

- **Purpose**: Specify the `.show` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.show`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Display session content, or a project overview. When `session_id::` is given without `project::`, `scope::`-resolved projects (default `local` — the current project and all its topic variants, e.g. `--commit`, `--default-topic`) are searched for that session. Without `session_id::`, resolves to the current (or given) project and shows a compact overview — counts, timestamp range, and the last `tail::` messages (default `10`) from its most-recently-active session — not a full per-session listing; pass `detail::sessions` to also enumerate every session. Use this when you need the content of a conversation, or a quick-glance summary of a project's activity.

**Parameters:** `session_id::`, `project::`, `show_entries::`, `show_metadata::`, `show_stat::`, `show_tokens::`, `scope::`, `path::`, `detail::`, `tail::`

`scope::` (default `local`) and `path::` (default cwd) narrow the session lookup used when `session_id::` is given without `project::` (see [Scope Configuration](../param_group/05_scope_configuration.md)) — `local` reproduces the original cwd-project-and-topic-variants lookup exactly. An explicit `project::`, or omitting `session_id::` entirely, makes both parameters a no-op.

`detail::` (default `projects`) and `tail::` (default `10`) apply only to the project-overview branches — when `session_id::` is absent (see [`detail::`](../param/30_detail.md), [`tail::`](../param/25_tail.md)). `detail::projects` shows the summary block and last `tail::` messages only; `detail::sessions` additionally appends the full per-session list. `tail::0` shows all messages from the most-recently-active session instead of capping at 10. Both are no-ops when `session_id::` is given.

**Exit:** `0` success | `1` error (invalid arguments, storage read failure, or no project in cwd)

**Syntax:**
```bash
claude_storage .show
claude_storage .show project::PROJECT
claude_storage .show project::PROJECT [detail::sessions] [tail::N]
claude_storage .show session_id::ID
claude_storage .show session_id::ID [show_entries::1] [show_metadata::1]
claude_storage .show session_id::ID project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id::` | [`SessionId`](../type/09_session_id.md) | optional | — | Session to display; when given without `project::`, `scope::`-resolved projects are searched (default `local`) |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | current dir | Project identifier; when given with `session_id::`, restricts search to this project only (scope::/path:: ignored) |
| `show_entries::` | Boolean | optional | `0` | Render the entry window as a raw UUID/type/timestamp list instead of formatted content |
| `show_metadata::` | Boolean | optional | `0` | Show metadata only (suppresses content) |
| `show_stat::` | Boolean | optional | `0` | Accepted for backward compatibility; no effect on output |
| `show_tokens::` | Boolean | optional | `0` | Include token usage section |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Project search boundary when `session_id::` is given without `project::` |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | current dir | Anchor path for `scope::` resolution |
| `detail::` | [`DetailLevel`](../type/14_detail_level.md) | optional | `projects` | Project-overview verbosity: summary only, or also enumerate every session (no effect when `session_id::` given) |
| `tail::` | Integer | optional | `10` | Trailing messages shown from the most-recently-active session in project-overview (no effect when `session_id::` given) |

`session_id::` and `project::` belong to [Session Identification](../param_group/03_session_identification.md) and [Project Scope](../param_group/02_project_scope.md) groups. `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md) and narrow the session lookup as described above when `session_id::` is given without `project::`. `show_stat::` and `show_tokens::` belong to the [Output Control group](../param_group/01_output_control.md). `detail::` and `tail::` are not members of a parameter group (see their own param pages for per-command defaults).

**Algorithm (5 steps):**
1. Parse and validate parameters — reject whitespace-only `session_id::`
2. Dispatch by parameter combination — (a) no params → cwd project overview, (b) `session_id::` only → search `scope::`-resolved projects (default `local`, reproducing the cwd-project-and-topic-variants lookup), (c) `project::` only → that project's overview, (d) both → that session in that project (scope::/path:: ignored)
3. Load project/session data — prefix matching for partial UUIDs (Git-style 8-char prefix)
4. Format project-overview output (branches a/c) — compact key:val summary block (path, storage dir, session counts, total entries, first/last timestamp) followed by the last `tail::` messages (default `10`) from the most-recently-active session; `show_entries::1` renders that window as a raw UUID/type/timestamp list instead of formatted content; `detail::sessions` additionally appends the full per-session list (default `detail::projects` omits it)
5. Format session-detail output (branches b/d) — metadata mode (`show_metadata::1`: structured fields, optionally + raw entries list via `show_entries::1`) or content mode (default: key:val attribute block, then full conversation chat-log); `show_stat::1` has no effect; token usage appended via `show_tokens::1`

**Examples:**
```bash
# Show current project's overview (summary + last 10 messages)
claude_storage .show

# Show a specific project's overview with the full session list
claude_storage .show project::/path/to/project detail::sessions

# Show the last 25 messages instead of the default 10
claude_storage .show tail::25

# Show a specific session — searches the current project and its topic variants (scope::local, the default)
claude_storage .show session_id::-default_topic

# Show session metadata only (no content)
claude_storage .show session_id::abc123 show_metadata::1

# Show a session in a specific project only (skips scope::/path:: resolution)
claude_storage .show session_id::ID project::/path/to/project

# Show a session anywhere in storage (scope::global)
claude_storage .show session_id::abc123 scope::global
```

**Notes:**
- When `session_id::` is given without `project::`, `scope::`-resolved projects are searched (default `local` — the current project and all its topic variants, reproducing the original lookup exactly); use `scope::global`/`under`/`relevant`/`around` with `path::` to broaden the search boundary. Supplying `project::` restricts lookup to one specific project and makes `scope::`/`path::` a no-op.
- Without `session_id::`, resolves to current directory (or given `project::`) project; exits with `1` if cwd has no project in storage
- Project-overview branches (a/c) default to `detail::projects` (summary block + last `tail::` messages only, `tail::` default `10`) rather than enumerating every session; pass `detail::sessions` to also list every session, or `tail::0` to show all messages from the most-recently-active session instead of capping at 10
- `show_metadata::1` selects metadata-only mode; `show_entries::1` appends a raw UUID/type/timestamp entries list within that mode, and — in project-overview branches — renders the `tail::`-windowed message view the same way instead of formatted content. It remains a no-op in session-detail content mode (branches b/d without `show_metadata::1`), which always shows full formatted entry content regardless.
- `show_stat::1` has no effect in any mode — session-detail metadata mode, session-detail content mode, and the project-overview summary block all already show the equivalent counts and timestamp range unconditionally
- A session file containing a malformed JSONL line no longer breaks `.show` (BUG-489): the corrupted line is skipped and the rest of the session's stats are computed normally. Before the fix, one bad line in one session's file could abort the entire project-level overview (case (a) above), not just that session's own row.
- `session_id::` matches a leading prefix of the session ID, never a substring found elsewhere in the ID — a matching predicate shared with `.export`/`.search`/`.tail` that briefly matched substrings anywhere in the ID (risking a silent match on the wrong session) is fixed (BUG-490)

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
| 25 | [`tail::`](../param/25_tail.md) | Integer | optional |
| 30 | [`detail::`](../param/30_detail.md) | [`DetailLevel`](../type/14_detail_level.md) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
