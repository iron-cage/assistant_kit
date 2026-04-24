# Commands Reference

All commands for the `claude_storage` CLI. Parameters use `param::value` syntax. All commands are read-only except `.session.ensure`, which creates the session working directory on disk.

See [params.md](params.md) for full parameter specs and [types.md](types.md) for type definitions.

## Commands Table

| # | Command | Status | Purpose | Params |
|---|---------|--------|---------|--------|
| 1 | `.status` | stable | Show storage overview and statistics | 2 |
| 2 | `.list` | stable | List projects or sessions | 8 |
| 3 | `.show` | stable | Display session or project details | 7 |
| 4 | `.count` | stable | Fast counting of items | 5 |
| 5 | `.search` | stable | Search session content by query | 8 |
| 6 | `.export` | stable | Export session to file | 6 |
| 7 | `.projects` | stable | Scoped project list with per-project conversation listing | 6 |
| 8 | `.path` | stable | Compute Claude storage path for a directory | 2 |
| 9 | `.exists` | stable | Check conversation history exists (exits 1 when absent) | 2 |
| 10 | `.session.dir` | stable | Compute session working directory path | 2 |
| 11 | `.session.ensure` | stable | Ensure session directory exists, report resume strategy | 3 |

**Total:** 11 commands (11 stable ✅, 0 deprecated)

---

### Command :: 1. `.status`

Show Claude Code storage overview and statistics. Use this when you need a global count of projects and sessions, or want to verify the storage root location.

**Parameters:** `path::`, `verbosity::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .status
claude_storage .status verbosity::2
claude_storage .status path::/custom/storage verbosity::3
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](types.md#storagepath) | optional | `~/.claude/` | Storage root override |
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | optional | `1` | Output detail level |

See [Output Control group](parameter_groups.md#output-control) for `verbosity` semantics.

**Verbosity output levels:**
- `0` — machine-readable counts only (`projects: N, sessions: N`)
- `1` — summary table with project and session totals (default)
- `2` — adds per-project session counts and user/assistant entry breakdowns

**Examples:**
```bash
# Default storage summary
claude_storage .status
# Output: summary table with project/session totals

# Detailed per-project breakdown
claude_storage .status verbosity::2
# Output: summary plus per-project session and entry counts
```

**Notes:**
- Default storage path is `~/.claude/`; override with `CLAUDE_STORAGE_ROOT` env var
- `verbosity::0` is suitable for piping to other tools

---

### Command :: 2. `.list`

List projects or conversations in Claude Code storage. Project-first view: all projects are listed, with conversations optionally shown per project. Use this when navigating projects or filtering by project path.

**Parameters:** `type::`, `path::`, `sessions::`, `session::`, `agent::`, `min_entries::`, `verbosity::`, `scope::`, `project::`, `count::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .list
claude_storage .list type::uuid
claude_storage .list path::SUBSTR [sessions::1]
claude_storage .list session::FILTER [agent::0|1] [min_entries::N]
claude_storage .list scope::relevant
claude_storage .list type::conversation project::PROJECT
claude_storage .list type::conversation count::1 project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `type::` | [`ProjectType`](types.md#projecttype) | optional | `all` | Project naming filter (`uuid`, `path`, `all`, `conversation`) |
| `path::` | [`PathSubstring`](types.md#pathsubstring) | optional | — | Filter projects by path substring |
| `sessions::` | Boolean | optional | `0` | Show sessions per project |
| `session::` | [`SessionFilter`](types.md#sessionfilter) | optional | — | Filter sessions by ID substring |
| `agent::` | Boolean | optional | — | Session type filter (`0`=main, `1`=agent) |
| `min_entries::` | [`EntryCount`](types.md#entrycount) | optional | — | Minimum entry count threshold |
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | optional | `1` | Output detail level |
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `global` | Project discovery boundary |
| `project::` | String | required for `type::conversation` | — | Project ID; scopes conversation listing |
| `count::` | Boolean | optional | `0` | Output only the count as a bare integer |

Session filter parameters belong to the [Session Filter group](parameter_groups.md#session-filter). See [Output Control group](parameter_groups.md#output-control) for `verbosity` semantics. See [Scope Configuration group](parameter_groups.md#scope-configuration) for `scope::` semantics.

**Examples:**
```bash
# List all projects
claude_storage .list

# List all sessions for projects matching path
claude_storage .list path::assistant sessions::1

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
- `session::`, `agent::`, or `min_entries::` auto-enables `sessions::1`; use `sessions::0` to suppress
- `type::uuid` shows projects identified by UUID rather than path encoding
- `type::conversation` requires `project::` and lists one conversation ID per line
- `count::1` with `type::conversation` outputs only the count as a bare integer (useful for scripting)
- `scope::global` is the default — lists all projects regardless of cwd; `scope::relevant` lists only projects in the ancestor chain of cwd

---

### Command :: 3. `.show`

Display session or project details. Scope-aware: when `session_id::` is given without `project::`, the current project and all its topic variants (`--commit`, `--default-topic`, etc.) are searched (scope::local). Without `session_id::`, resolves to the current project. Use this when you need the content of a conversation or a project's session list.

**Parameters:** `session_id::`, `project::`, `verbosity::`, `entries::`, `metadata::`, `scope::`, `path::`

**Exit:** `0` success | `1` argument error | `2` storage read error or project not found

**Syntax:**
```bash
claude_storage .show
claude_storage .show session_id::ID
claude_storage .show project::PROJECT
claude_storage .show session_id::ID [entries::1] [metadata::1]
claude_storage .show session_id::ID project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id::` | [`SessionId`](types.md#sessionid) | optional | — | Session to display; when given without `project::`, the current project and all its topic variants are searched (scope::local) |
| `project::` | [`ProjectId`](types.md#projectid) | optional | current dir | Project identifier; when given with `session_id::`, restricts search to this project only |
| `entries::` | Boolean | optional | `0` | Show all entries in session |
| `metadata::` | Boolean | optional | `0` | Show metadata only (suppresses content) |
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | optional | `1` | Output detail level |
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `local` | Project search boundary (Case 2 only: session_id without project::) |
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Scope anchor path |

`session_id::` and `project::` belong to [Session Identification](parameter_groups.md#session-identification) and [Project Scope](parameter_groups.md#project-scope) groups. `scope::` and `path::` belong to the [Scope Configuration group](parameter_groups.md#scope-configuration).

**Examples:**
```bash
# Show current project's session list
claude_storage .show

# Show a specific session — searches all projects globally (no project:: needed)
claude_storage .show session_id::-default_topic

# Show session metadata only (no content)
claude_storage .show session_id::abc123 metadata::1

# Show a session in a specific project only (skips global search)
claude_storage .show session_id::ID project::/path/to/project
```

**Notes:**
- When `session_id::` is given without `project::`, the current project and all its topic variants (scope::local) are searched; supply `project::` to restrict lookup to one specific project
- Without `session_id::`, resolves to current directory project; exits with `2` if cwd has no project in storage
- `entries::1` and `metadata::1` are mutually exclusive; `entries::1` takes precedence

---

### Command :: 4. `.count`

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
| `target::` | [`TargetType`](types.md#targettype) | optional | `projects` | What to count (`projects`, `sessions`, `entries`, `conversations`) |
| `project::` | [`ProjectId`](types.md#projectid) | optional | — | Scope to this project |
| `session::` | [`SessionId`](types.md#sessionid) | optional | — | Scope to this session |
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `global` | Count boundary |
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Scope anchor path |

`project::` belongs to the [Project Scope group](parameter_groups.md#project-scope). `scope::` and `path::` belong to the [Scope Configuration group](parameter_groups.md#scope-configuration).

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

---

### Command :: 5. `.search`

Search session content for a query string across projects and sessions. Use this to find conversations by topic, code fragment, or any text that appeared in a session.

**Parameters:** `query::`, `project::`, `session::`, `case_sensitive::`, `entry_type::`, `verbosity::`, `scope::`, `path::`

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
| `project::` | [`ProjectId`](types.md#projectid) | optional | — | Restrict to this project |
| `session::` | [`SessionId`](types.md#sessionid) | optional | — | Restrict to this session |
| `case_sensitive::` | Boolean | optional | `0` | Enable case-sensitive matching |
| `entry_type::` | [`EntryType`](types.md#entrytype) | optional | `all` | Filter by entry type |
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | optional | `1` | Output detail level |
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `global` | Search boundary |
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Scope anchor path |

`project::` belongs to the [Project Scope group](parameter_groups.md#project-scope). `scope::` and `path::` belong to the [Scope Configuration group](parameter_groups.md#scope-configuration).

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

---

### Command :: 6. `.export`

Export a session to a file in the specified format. Use this to save a conversation for sharing, archiving, or further processing.

**Parameters:** `session_id::`, `output::`, `format::`, `project::`, `scope::`, `path::`

**Exit:** `0` success | `1` argument error (missing required params) | `2` storage read error or write error

**Syntax:**
```bash
claude_storage .export session_id::ID output::PATH
claude_storage .export session_id::ID output::PATH format::FORMAT
claude_storage .export session_id::ID output::PATH project::PROJECT
claude_storage .export session_id::ID output::PATH scope::global
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id::` | [`SessionId`](types.md#sessionid) | **required** | — | Session to export |
| `output::` | [`StoragePath`](types.md#storagepath) | **required** | — | Output file path |
| `format::` | [`ExportFormat`](types.md#exportformat) | optional | `markdown` | Export format |
| `project::` | [`ProjectId`](types.md#projectid) | optional | current dir | Source project |
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `local` | Project search boundary |
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Scope anchor path |

`session_id::` belongs to [Session Identification](parameter_groups.md#session-identification). `project::` belongs to [Project Scope](parameter_groups.md#project-scope). `scope::` and `path::` belong to the [Scope Configuration group](parameter_groups.md#scope-configuration).

**Examples:**
```bash
# Export as markdown (default)
claude_storage .export session_id::-default_topic output::conversation.md

# Export as JSON for programmatic use
claude_storage .export session_id::abc123 format::json output::session.json

# Export as plain text
claude_storage .export session_id::-default_topic format::text output::transcript.txt

# Export a session found anywhere in storage
claude_storage .export session_id::ID output::PATH scope::global
```

**Notes:**
- Both `session_id::` and `output::` are required; command exits with `1` if either is missing
- Output file is overwritten without warning if it already exists

---

### Command :: 7. `.projects`

Project list with scope control; conversations are grouped by project directory and one entry is shown per project (not per session file). Bare invocation shows all projects in the bidirectional neighborhood (ancestors + current + descendants via `scope::around`).

**Parameters:** `scope::`, `path::`, `session::`, `agent::`, `min_entries::`, `limit::`, `verbosity::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .projects
claude_storage .projects scope::around
claude_storage .projects scope::relevant
claude_storage .projects scope::under path::PATH
claude_storage .projects scope::global [agent::1] [min_entries::N]
claude_storage .projects limit::5
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope::` | [`ScopeValue`](types.md#scopevalue) | optional | `around` | Session discovery scope |
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Base path for scope resolution |
| `session::` | [`SessionFilter`](types.md#sessionfilter) | optional | — | Filter sessions by ID substring |
| `agent::` | Boolean | optional | — | Session type filter (`0`=main, `1`=agent) |
| `min_entries::` | [`EntryCount`](types.md#entrycount) | optional | — | Minimum entry count threshold |
| `limit::` | Integer | optional | `0` | Max main sessions per project at v1 (`0` = unlimited) |
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | optional | `1` | Output detail level |

`scope::` and `path::` belong to the [Scope Configuration group](parameter_groups.md#scope-configuration). Session filters belong to [Session Filter](parameter_groups.md#session-filter).

**Default invocation:**

Bare `clg .projects` uses `scope::around` — showing all projects in the bidirectional neighborhood of cwd (ancestors upward to `/` plus all descendants). Output format is the same as any explicit invocation (see Verbosity output format below). No sessions in scope → `No active project found.`

**Examples:**
```bash
# Neighborhood view — ancestors + current + descendants (default)
claude_storage .projects

# Explicit bidirectional neighborhood
claude_storage .projects scope::around

# All sessions related to current work (ancestor chain)
claude_storage .projects scope::relevant

# All sessions under a subtree
claude_storage .projects scope::under path::/home/user1/pro

# All sessions, agent only, with entries
claude_storage .projects scope::global agent::1 min_entries::50

# Show at most 5 sessions per project
claude_storage .projects scope::global limit::5
```

**Notes:**
- `scope::relevant` walks UP from cwd to `/`, collecting sessions from every project at each ancestor level
- Distinct from `.exists`: that checks existence (exit 0/1); this lists conversations
- **Fixed (issue-024)**: `scope::local/relevant/under` previously returned 0 results when the base path contained underscores (e.g., `wip_core`). Root cause: lossy encoding mapped `_` and `/` identically; decoded paths diverged from real paths. Fixed by comparing encoded paths directly against raw storage directory names.
- **Fixed (issue-029)**: `scope::under` (and all scopes at verbosity ≥ 1) previously displayed project path headers with underscore-named directories split as path separators (e.g., `wip_core` → `wip/core`). Root cause: `decode_project_display` heuristic defaulted to `/` for every `-` boundary; underscore-named dirs were indistinguishable from path separators in the encoded form. Fixed by adding a filesystem-guided fallback that walks the real directory tree to resolve ambiguous boundaries.
- **Fixed (issue-030)**: Session path headers previously showed only the base directory, truncating hyphen-prefixed topic components (e.g., `src/-default_topic` was shown as `src`). Root cause: `decode_project_display` stripped all `--topic` suffixes before decoding. Fixed by decoding the base path with filesystem guidance (resolves `_` vs `/` ambiguity per issue-029), then appending topic components as hyphen-prefixed directory names. **Display-path invariant**: topic components must always be appended regardless of whether the directory currently exists on disk — the storage key encodes the actual CWD at session time and must be decoded as-is.
- **Fixed (issue-035)**: The issue-030 fix introduced an incorrect filesystem existence check — topic components were only appended when `candidate.exists()` was true. Sessions recorded in `dir/-commit` displayed as `dir` after the `-commit` directory was deleted, obscuring which working directory the session used. Root cause: `decode_project_display` called `candidate.exists()` and broke at the first missing topic dir. Fixed by removing the existence guard from the topic-extension loop; all topic components are always appended unconditionally — filesystem state at query time must not affect which CWD a session is attributed to. (Task 025.)
- **Fixed (issue-031)**: `scope::under` previously included sessions from sibling modules whose names start with the base name followed by `_` (e.g., `claude_storage_core` matched when base was `claude_storage`). Root cause: `encode_path` maps both `_` and `/` to `-`, so string `starts_with` cannot distinguish a child path from an underscore-suffixed sibling — both produce the same encoded prefix. Fixed by a two-stage predicate: string prefix is fast-reject only; `decode_path_via_fs` + `Path::starts_with` (component-wise) provides correct disambiguation.
- **Fixed (issue-032)**: `scope::relevant` previously included sessions from sibling projects whose encoded name is a string prefix of the current path's encoded form (e.g., `/base` matched when current path was `/base_extra`). Root cause: `is_relevant_encoded` used `encoded_base.starts_with(dir_name + "-")` which cannot distinguish a true ancestor (`base/sub`) from a same-level sibling with an underscore suffix (`base_extra`). Fixed by the same two-stage predicate as issue-031: `decode_path_via_fs` + `base_path.starts_with(decoded_path)` (component-wise) for disambiguation.

**Verbosity output format:**

All invocations (including bare) use the same list output format. Output is grouped by project at verbosity ≥ 1. Path header is always shown (never suppressed):

```
Found N projects:

~/path/to/project-a: (2 conversations, 12 agents)
  * a1b2c3d4  2h ago  (347 entries)  [8 agents: 5×Explore, 2×general-purpose, 1×Plan]
  - e5f6a7b8  1d ago  (42 entries)   [4 agents: 3×Explore, 1×general-purpose]

~/path/to/project-b: (1 session)
  * c9d0e1f2  3d ago  (2 entries)
```

Family display: agents are grouped by parent session into families. Each root
session line shows an inline `[N agents: breakdown]` suffix. Roots with no
agents show no bracket suffix. Orphan families (root deleted) use `?` marker.
When `agent::` filter is set, family grouping is disabled — flat display.

At `verbosity::2+`, agents are tree-indented under their parent:
```
~/path/to/project-a: (2 conversations, 12 agents)
  - a1b2c3d4-e5f6-7890-abcd-ef1234567890  (347 entries)
    ├─ agent-a6061d6e2a0c37a78  Explore  12 entries
    ├─ agent-3f8b2c91ea44d2b10  Explore   8 entries
    └─ agent-7e4a0b23ff129c5a2  general-purpose  42 entries
  - e5f6a7b8-...  (42 entries)
    └─ agent-c1d2e3f4  Explore  15 entries
```

**Verbosity matrix:**

| Verbosity | Project output | Session lines | Agent sessions | Mtime | Entry count | Sort |
|-----------|----------------|---------------|----------------|-------|-------------|------|
| 0 | Project paths only (one per line, machine-readable) | — | — | — | — | mtime desc |
| 1 (default) | `~/path: (N conversations, M agents)` | `  * {short-id}  {mtime}  ({n} entries)  [N agents: breakdown]` | family-grouped per parent | ✓ | ✓ | mtime desc |
| 2+ | `~/path: (N conversations, M agents)` | `  - {full-id}  ({n} entries)` | tree-indented under parent (`├─`/`└─`) | — | ✓ | mtime desc |

**v1 display rules:**
- `*` marks the first (most recent) root session; `-` marks the rest
- Short UUID: 36-char UUID IDs are truncated to first 8 chars; non-UUID IDs shown in full
- Zero-byte sessions excluded (startup placeholders, B8)
- Family display: agents grouped by parent; inline `[N agents: N×Type, …]` per root
- Orphan families (no root): `  ? (orphan)  [N agents: breakdown]`
- `limit::N` caps families per project; truncated projects show `... and N more sessions` hint

- `verbosity::0` — project paths only (one per line, machine-readable); suitable for piping
- `verbosity::1` — `Found N projects:` header; grouped per project with family display; project header always shows `(N conversations)` or `(N conversations, M agents)` when agents present
- `verbosity::2+` — same grouping; agents tree-indented under parent; full IDs; entry count per session

---

### Command :: 8. `.path`

Compute the Claude Code storage path for a directory without requiring it to exist. Use this to inspect what storage path would be used for a given working directory.

**Parameters:** `path::`, `topic::`

**Exit:** `0` success | `1` argument error (invalid path or topic)

**Syntax:**
```bash
claude_storage .path
claude_storage .path path::PATH
claude_storage .path topic::TOPIC
claude_storage .path path::PATH topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Directory to compute storage path for |
| `topic::` | [`TopicName`](types.md#topicname) | optional | — | Session topic suffix (without leading `-`) |

**Output:** Single line — the absolute path to `~/.claude/projects/{encoded}/` (or `{encoded}-{topic}/` when `topic::` given).

**Examples:**
```bash
# Storage path for current directory
claude_storage .path

# Storage path for a specific directory
claude_storage .path path::/home/user/project

# Storage path with topic suffix
claude_storage .path topic::default_topic

# Storage path with directory and topic
claude_storage .path path::~/pro/lib/myapp topic::work
```

**Notes:**
- The returned path does not need to exist on disk
- Use `.exists` to test whether the path has conversation history

---

### Command :: 9. `.exists`

Check whether a directory has Claude Code conversation history. Exits with code `1` when no history is found, making it ideal for shell conditional logic.

**Parameters:** `path::`, `topic::`

**Exit:** `0` history found | `1` no history found

**Syntax:**
```bash
claude_storage .exists
claude_storage .exists path::PATH
claude_storage .exists topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](types.md#storagepath) | optional | cwd | Directory to check |
| `topic::` | [`TopicName`](types.md#topicname) | optional | — | Session topic suffix (without leading `-`) |

**Output:**
- Exit 0: `"sessions exist\n"` on stdout
- Exit 1: `"no sessions"` on stderr

**Examples:**
```bash
# Check current directory
claude_storage .exists

# Check specific directory
claude_storage .exists path::/home/user/project

# Shell conditional
if clg .exists; then echo "Has history"; else echo "Fresh start"; fi
```

**Notes:**
- Exit code `1` is an informational result (no history found), not a command error
- This is the sole history-check command; `.session` was removed as a duplicate (task-014)

---

### Command :: 10. `.session.dir`

Compute the session working directory path (`{base}/-{topic}`) without creating it. Use this to determine the correct session directory before deciding whether to start or resume.

**Parameters:** `path::`, `topic::`

**Exit:** `0` success | `1` argument error (missing required `path::`, invalid topic)

**Syntax:**
```bash
claude_storage .session.dir path::PATH
claude_storage .session.dir path::PATH topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](types.md#storagepath) | **required** | — | Base directory |
| `topic::` | [`TopicName`](types.md#topicname) | optional | `default_topic` | Session topic (without leading `-`) |

**Output:** Single line — the absolute path to `{base}/-{topic}`.

**Examples:**
```bash
# Session dir for current directory with default topic
claude_storage .session.dir path::.

# Session dir for specific project
claude_storage .session.dir path::/home/user/project

# Session dir with custom topic
claude_storage .session.dir path::/home/user/project topic::work
```

**Notes:**
- `path::` is required; omitting it returns exit 1
- The returned directory path does not need to exist on disk
- Use `.session.ensure` to create the directory and detect resume strategy

---

### Command :: 11. `.session.ensure`

Ensure a session working directory exists, creating it if necessary. Reports whether the session has existing conversation history (`resume`) or is starting fresh (`fresh`). Outputs two lines: the absolute session directory path and the strategy.

**Parameters:** `path::`, `topic::`, `strategy::`

**Exit:** `0` success | `1` argument error (missing required `path::`, invalid params)

**Syntax:**
```bash
claude_storage .session.ensure path::PATH
claude_storage .session.ensure path::PATH topic::TOPIC
claude_storage .session.ensure path::PATH strategy::resume|fresh
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](types.md#storagepath) | **required** | — | Base directory |
| `topic::` | [`TopicName`](types.md#topicname) | optional | `default_topic` | Session topic (without leading `-`) |
| `strategy::` | [`StrategyType`](types.md#strategytype) | optional | auto-detect | Override resume strategy |

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
- `path::` is required; omitting it returns exit 1
- When `strategy::resume` is forced but no history exists, the output still reports `resume` (caller's intent is respected)

