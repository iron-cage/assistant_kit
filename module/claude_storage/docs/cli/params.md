# Parameters Reference

All parameters for the `claude_storage` CLI. Parameters use `param::value` syntax.

See [types.md](types.md) for type definitions and [parameter_groups.md](parameter_groups.md) for related parameter sets.

## Parameters Table

| # | Parameter | Type | Default | Commands | Purpose |
|---|-----------|------|---------|----------|---------|
| 1 | `agent::` | Boolean | — | 2 | Session type filter (main vs agent) |
| 2 | `case_sensitive::` | Boolean | `0` | 1 | Case-sensitive search matching |
| 3 | `entries::` | Boolean | `0` | 1 | Show all session entries |
| 4 | `entry_type::` | [`EntryType`](types.md#entrytype) | `all` | 1 | Filter search by entry type |
| 5 | `format::` | [`ExportFormat`](types.md#exportformat) | `markdown` | 1 | Export output format |
| 6 | `metadata::` | Boolean | `0` | 1 | Show metadata only mode |
| 7 | `min_entries::` | [`EntryCount`](types.md#entrycount) | — | 2 | Minimum entry count threshold |
| 8 | `output::` | [`StoragePath`](types.md#storagepath) | — | 1 | Export output file path |
| 9 | `path::` | varies | varies | 8 | Path argument (semantics vary by command) |
| 10 | `project::` | [`ProjectId`](types.md#projectid) | current dir | 5 | Project scope identifier |
| 11 | `query::` | String | — | 1 | Search query string |
| 12 | `scope::` | [`ScopeValue`](types.md#scopevalue) | varies | 6 | Session/project discovery scope |
| 13 | `session::` | [`SessionFilter`](types.md#sessionfilter) / [`SessionId`](types.md#sessionid) | — | 4 | Session filter (listing) or scope pin (count/search) |
| 14 | `session_id::` | [`SessionId`](types.md#sessionid) | — | 2 | Direct session identifier |
| 15 | `sessions::` | Boolean | `0` | 1 | Explicit session display toggle |
| 16 | `target::` | [`TargetType`](types.md#targettype) | `projects` | 1 | Count operation target |
| 17 | `topic::` | [`TopicName`](types.md#topicname) | — | 5 | Session topic suffix (without leading `-`) |
| 18 | `type::` | [`ProjectType`](types.md#projecttype) | `all` | 1 | Project naming scheme filter |
| 19 | `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | `1` | 5 | Output detail level |
| 20 | `strategy::` | [`StrategyType`](types.md#strategytype) | auto-detect | 1 | Resume strategy override for `.session.ensure` |
| 21 | `count::` | Boolean | `0` | 1 | Output count only instead of full list (`.list type::conversation`) |

**Total:** 21 parameters

---

### Parameter :: 1. `agent::`

Session type filter for listing operations.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`, unset
- `0` = main sessions only (no agent sessions)
- `1` = agent sessions only (no main sessions)
- Unset = all session types (no filter)

**Default:** unset (all session types shown)

**Commands:** `.list`, `.projects`
(See [commands.md#command--2-list](commands.md#command--2-list) and [commands.md#command--7-projects](commands.md#command--7-projects))

**Purpose:** Distinguishes between main conversation sessions and agent sub-sessions spawned by tool calls. Agent sessions are stored as `agent-*.jsonl` files and have `isSidechain: true`. Use `agent::1` to inspect sub-agent behavior, `agent::0` to see only top-level conversations.

**Side effect:** Auto-enables `sessions::1` in `.list`.

**Examples:**
```bash
# Valid values
agent::0       # Main sessions only
agent::1       # Agent sessions only
               # (unset) — all session types

# Invalid values (rejected with error)
agent::2       # Not a boolean: "agent must be 0 or 1"
agent::yes     # Not a boolean: "agent must be 0 or 1"
```

**Group:** [Session Filter](parameter_groups.md#session-filter)

---

### Parameter :: 2. `case_sensitive::`

Enable case-sensitive matching in search operations.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = case-insensitive (default)
- `1` = case-sensitive

**Default:** `0` (case-insensitive)

**Commands:** `.search`
(See [commands.md#command--5-search](commands.md#command--5-search))

**Purpose:** Controls whether search matches are case-sensitive. Default case-insensitive mode is practical for most searches; enable case-sensitive when searching for identifiers, variable names, or other case-significant strings.

**Examples:**
```bash
# Valid values
case_sensitive::0     # Case-insensitive (default)
case_sensitive::1     # Case-sensitive

# Invalid values
case_sensitive::true  # Not a boolean: "case_sensitive must be 0 or 1"
```

---

### Parameter :: 3. `entries::`

Show all individual entries in a session display.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = summary view (default)
- `1` = show all entry records

**Default:** `0`

**Commands:** `.show`
(See [commands.md#command--3-show](commands.md#command--3-show))

**Purpose:** When enabled, lists every entry in the session with UUID, type, and timestamp, rather than showing conversation content. Useful for inspecting session structure or counting messages without loading full content.

**Examples:**
```bash
entries::0    # Summary view
entries::1    # Full entry listing
```

---

### Parameter :: 4. `entry_type::`

Filter search results by conversation entry type.

**Type:** [`EntryType`](types.md#entrytype)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `user`, `assistant`, `all`
- Case-insensitive on input
- Error on invalid: `"entry_type must be user|assistant|all, got {value}"`

**Default:** `all`

**Commands:** `.search`
(See [commands.md#command--5-search](commands.md#command--5-search))

**Purpose:** Restricts search to only user-authored messages or only assistant-authored messages. Use `entry_type::user` when searching for what you asked about; use `entry_type::assistant` when searching for what the assistant responded with.

**Examples:**
```bash
# Valid values
entry_type::user        # User messages only
entry_type::assistant   # Assistant messages only
entry_type::all         # No filter (default)

# Invalid values (rejected with error)
entry_type::both        # "entry_type must be user|assistant|all, got both"
entry_type::system      # "entry_type must be user|assistant|all, got system"
```

---

### Parameter :: 5. `format::`

Export output format for `.export` operations.

**Type:** [`ExportFormat`](types.md#exportformat)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `markdown`, `json`, `text`
- Case-insensitive on input
- Error on invalid: `"format must be markdown|json|text, got {value}"`

**Default:** `markdown`

**Commands:** `.export`
(See [commands.md#command--6-export](commands.md#command--6-export))

**Purpose:** Selects the output serialization format. `markdown` produces a human-readable conversation document; `json` produces the raw JSONL entries as a JSON array suitable for programmatic processing; `text` produces a plain text transcript without markup.

**Examples:**
```bash
# Valid values
format::markdown   # Human-readable document (default)
format::json       # Raw entries as JSON array
format::text       # Plain text transcript

# Invalid values
format::html       # "format must be markdown|json|text, got html"
format::pdf        # "format must be markdown|json|text, got pdf"
```

---

### Parameter :: 6. `metadata::`

Show session metadata only, suppressing conversation content.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = show conversation content (default)
- `1` = show technical metadata only

**Default:** `0`

**Commands:** `.show`
(See [commands.md#command--3-show](commands.md#command--3-show))

**Purpose:** When enabled, displays session technical metadata (session ID, entry count, first/last timestamps, token usage) without loading or rendering conversation content. Useful for inspecting session characteristics without reading the full content.

**Examples:**
```bash
metadata::0    # Show content (default)
metadata::1    # Metadata only
```

---

### Parameter :: 7. `min_entries::`

Filter sessions by minimum entry count threshold.

**Type:** [`EntryCount`](types.md#entrycount)

**Fundamental Type:** Integer wrapper (non-negative)

**Constraints:**
- Must be integer ≥ 0
- Error on non-integer: `"min_entries must be a non-negative integer, got {value}"`
- Error on negative: `"min_entries must be ≥ 0, got {value}"`

**Default:** unset (no minimum)

**Commands:** `.list`, `.projects`
(See [commands.md#command--2-list](commands.md#command--2-list) and [commands.md#command--7-projects](commands.md#command--7-projects))

**Purpose:** Excludes sessions with fewer entries than the threshold. Useful for finding substantive conversations (skip one-message sessions) or for performance (only load sessions known to have content).

**Side effect:** Auto-enables `sessions::1` in `.list`.

**Examples:**
```bash
# Valid values
min_entries::0    # No minimum (includes all sessions)
min_entries::10   # At least 10 entries
min_entries::100  # Substantial sessions only

# Invalid values
min_entries::-1   # "min_entries must be ≥ 0, got -1"
min_entries::abc  # "min_entries must be a non-negative integer, got abc"
```

**Group:** [Session Filter](parameter_groups.md#session-filter)

---

### Parameter :: 8. `output::`

Output file path for export operations.

**Type:** [`StoragePath`](types.md#storagepath)

**Fundamental Type:** String (filesystem path)

**Constraints:**
- Must be a non-empty string
- Parent directory must exist (error if parent does not exist)
- File is overwritten without warning if it exists

**Default:** none — **required**

**Commands:** `.export`
(See [commands.md#command--6-export](commands.md#command--6-export))

**Purpose:** Specifies where the exported session content is written. Accepts absolute and `~`-prefixed paths.

**Examples:**
```bash
# Valid values
output::conversation.md              # Relative path
output::/home/user/exports/chat.md   # Absolute path
output::~/exports/session.json       # Home-relative path

# Error cases
output::                             # Empty path error
output::/nonexistent/dir/file.md     # Parent directory does not exist
```

---

### Parameter :: 9. `path::`

Path argument. Semantics differ by command — see command sections for exact behavior.

**Type:** [`StoragePath`](types.md#storagepath) or [`PathSubstring`](types.md#pathsubstring) depending on command

**Fundamental Type:** String

**Constraints:** Command-dependent (see table below)

**Default:** Command-dependent

**Commands:** `.status`, `.list`, `.projects`, `.count`, `.search`, `.show`, `.export`, `.path`, `.exists`, `.session.dir`, `.session.ensure`
(See individual command sections in [commands.md](commands.md))

**Per-command semantics:**

| Command | Type | Default | Semantics |
|---------|------|---------|-----------|
| `.status` | StoragePath | `~/.claude/` | Storage root override |
| `.list` | PathSubstring | — | Filter projects by path substring (case-insensitive) |
| `.projects` | StoragePath | cwd | Scope anchor path |
| `.count` | StoragePath | cwd | Scope anchor path |
| `.search` | StoragePath | cwd | Scope anchor path |
| `.show` | StoragePath | cwd | Scope anchor path |
| `.export` | StoragePath | cwd | Scope anchor path |
| `.path` | StoragePath | cwd | Directory to compute storage path for |
| `.exists` | StoragePath | cwd | Directory to check for history |
| `.session.dir` | StoragePath | — | Base directory (required) |
| `.session.ensure` | StoragePath | — | Base directory (required) |

**Purpose:** Provides a path context appropriate to each command. In `.exists`, `.path`, `.session.dir`, and `.session.ensure`, it is a filesystem path to process. In `.list`, it is a substring filter on project paths. In `.projects`, `.count`, `.search`, `.show`, and `.export`, it anchors the scope discovery when paired with `scope::`.

**Examples:**
```bash
# .status: storage root override
.status path::~/.claude/

# .list: path substring filter
.list path::assistant          # Matches all projects with "assistant" in path

# .exists: directory check
.exists path::/home/user/project

# .path: storage path computation
.path path::/home/user/project

# .session.dir / .session.ensure: base directory (required)
.session.dir path::/home/user/project
.session.ensure path::/home/user/project

# .projects / .count / .search / .show / .export: scope anchor
.projects scope::under path::/home/user1/pro
.count scope::under path::/home/user1/pro
.search query::error scope::under path::/home/user1/pro
```

**Group (scope anchor context):** [Scope Configuration](parameter_groups.md#scope-configuration) — `path::` acts as the scope anchor paired with `scope::` in `.projects`, `.count`, `.search`, `.show`, and `.export`; its role in `.status`, `.list`, `.exists`, `.path`, `.session.dir`, and `.session.ensure` is independent and not part of this group.

---

### Parameter :: 10. `project::`

Project identifier for scoping operations to a specific project.

**Type:** [`ProjectId`](types.md#projectid)

**Fundamental Type:** String (multi-format identifier)

**Constraints:**
- Accepts multiple formats (see type definition)
- Error if project not found: `"project not found: {value}"`

**Default:** resolves to the project for the current working directory

**Commands:** `.show`, `.count`, `.search`, `.export`
(See individual command sections in [commands.md](commands.md))

**Purpose:** Restricts an operation to a specific project. Without `project::`, most commands default to the current directory's project. Use `project::` when working with a project other than the current directory.

**Accepted formats:**
- Absolute path: `project::/home/user1/pro/lib/consumer`
- Path-encoded ID: `project::-home-user1-pro-lib-consumer`
- UUID: `project::8d795a1c-c81d-4010-8d29-b4e678272419`
- `Path(...)` form from `.list`: `project::Path("/home/user1/pro/lib/consumer")`

**Examples:**
```bash
# All equivalent — identify the same project
project::/home/user1/pro/lib/consumer
project::-home-user1-pro-lib-consumer
project::8d795a1c-c81d-4010-8d29-b4e678272419

# Invalid
project::               # Empty path error
project::nonexistent    # Project not found error
```

**Group:** [Project Scope](parameter_groups.md#project-scope)

---

### Parameter :: 11. `query::`

Search query string. Required by `.search`.

**Type:** String

**Fundamental Type:** String (raw)

**Constraints:**
- Must be non-empty
- Error on empty: `"query must be non-empty"`

**Default:** none — **required**

**Commands:** `.search`
(See [commands.md#command--5-search](commands.md#command--5-search))

**Alias:** `q`

**Purpose:** The text to search for in session content. Default matching is case-insensitive substring. Multi-word phrases should be quoted in the shell.

**Examples:**
```bash
# Valid values
query::error                        # Single term
query::"session management"         # Phrase (shell-quoted)
q::version_bump                     # Using alias

# Invalid values
query::                             # "query must be non-empty"
```

---

### Parameter :: 12. `scope::`

Discovery scope for session and project operations.

**Type:** [`ScopeValue`](types.md#scopevalue)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `relevant`, `local`, `under`, `global`, `around`
- Case-insensitive on input
- Error on invalid: `"scope must be relevant|local|under|global|around, got {value}"`

**Default:** varies by command (see table below)

**Commands:** `.list`, `.count`, `.search`, `.show`, `.export`, `.projects`
(See individual command sections in [commands.md](commands.md))

**Purpose:** Controls which projects are searched or counted. `local` is the narrowest (current project only); `global` is the broadest (all projects). `relevant` walks the ancestor chain from cwd upward to `/`; `under` descends into the subtree; `around` combines both for a full neighborhood view — models "what governs this work and what lives under it."

**Per-command semantics:**

| Command | Default | Scope semantics |
|---------|---------|-----------------|
| `.list` | `global` | Discovery boundary for project listing |
| `.count` | `global` | Boundary for what gets counted |
| `.search` | `global` | Boundary for what gets searched |
| `.show` | `local` | Project search boundary when no `project::` given |
| `.export` | `local` | Project search boundary for source session lookup |
| `.projects` | `around` | Session discovery scope (ancestors + current + descendants) |

**Examples:**
```bash
# Valid values
scope::local      # Current project only
scope::relevant   # All ancestor projects up to /
scope::under      # All projects under path
scope::global     # All projects in storage
scope::around     # Ancestors + current + descendants (default for .projects)

# Invalid values
scope::all        # "scope must be relevant|local|under|global|around, got all"
```

**Group:** [Scope Configuration](parameter_groups.md#scope-configuration)

---

### Parameter :: 13. `session::`

Session identifier parameter — acts as substring filter in listing commands, as exact identifier in counting/search commands.

**Type:** [`SessionFilter`](types.md#sessionfilter) (in `.list`, `.projects`) / [`SessionId`](types.md#sessionid) (in `.count`, `.search`)

**Fundamental Type:** String

**Constraints:**
- Non-empty string expected
- In `.list` and `.projects`: case-insensitive substring match against session filename stem
- In `.count` and `.search`: exact match (used to scope to a specific session)

**Default:** unset (no filter / no scope restriction)

**Commands:** `.list`, `.count`, `.search`, `.projects`
(See individual command sections in [commands.md](commands.md))

**Per-command semantics:**

| Command | Type | Semantics |
|---------|------|-----------|
| `.list` | SessionFilter | Substring filter — shows sessions whose ID contains this string |
| `.projects` | SessionFilter | Substring filter — shows sessions whose ID contains this string |
| `.count` | SessionId | Exact scope — counts entries within this specific session |
| `.search` | SessionId | Exact scope — restricts search to this specific session |

**Purpose:** Narrows results by session identity. In listing contexts (`.list`, `.projects`), acts as a substring filter for discovery. In counting/search contexts (`.count`, `.search`), acts as an exact scope pin to a specific session.

**Side effect:** Auto-enables `sessions::1` in `.list`.

**Examples:**
```bash
# Listing: substring filter
session::commit       # Matches -commit.jsonl, auto-commit.jsonl
session::default      # Matches -default_topic.jsonl

# Counting/search: exact scope
.count target::entries project::abc session::-default_topic
.search query::error session::-default_topic
```

**Group (listing context):** [Session Filter](parameter_groups.md#session-filter) — applies to `.list` and `.projects` only where `session::` acts as a substring filter alongside `agent::` and `min_entries::`.

---

### Parameter :: 14. `session_id::`

Direct session identifier for single-session operations.

**Type:** [`SessionId`](types.md#sessionid)

**Fundamental Type:** String (filename stem)

**Constraints:**
- Non-empty string
- Error if session not found: `"session not found: {value}"`

**Default:** none (optional in `.show`, required in `.export`)

**Commands:** `.show`, `.export`
(See [commands.md#command--3-show](commands.md#command--3-show) and [commands.md#command--6-export](commands.md#command--6-export))

**Purpose:** Identifies a specific session by its filename stem (JSONL filename without `.jsonl`). Required for `.export`; optional for `.show` (omitting it shows the project instead).

**Examples:**
```bash
# Named sessions
session_id::-default_topic
session_id::-commit

# UUID sessions
session_id::8d795a1c-c81d-4010-8d29-b4e678272419
```

**Group:** [Session Identification](parameter_groups.md#session-identification)

---

### Parameter :: 15. `sessions::`

Explicit control over session display in `.list`.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = suppress session display (even when session filters are active)
- `1` = force session display (even with no session filters)
- Auto-enabled by `session::`, `agent::`, or `min_entries::`

**Default:** `0` (auto-detection active)

**Commands:** `.list`
(See [commands.md#command--2-list](commands.md#command--2-list))

**Purpose:** Normally session display is auto-controlled: the presence of any session filter enables it. `sessions::` provides an explicit override — `sessions::0` suppresses display even when filters are set (useful for counting projects that have matching sessions), and `sessions::1` forces display even with no filters.

**Examples:**
```bash
sessions::0    # Force off (suppress even when filters active)
sessions::1    # Force on (show even with no filters)
               # (unset) — auto-detect from other params
```

---

### Parameter :: 16. `target::`

Specifies what `.count` should count.

**Type:** [`TargetType`](types.md#targettype)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `projects`, `sessions`, `entries`, `conversations`
- Case-insensitive on input
- Error on invalid: `"target must be projects|sessions|entries|conversations, got {value}"`

**Default:** `projects`

**Commands:** `.count`
(See [commands.md#command--4-count](commands.md#command--4-count))

**Purpose:** Selects the granularity of counting. `projects` counts all projects in storage. `sessions` counts sessions within a project. `entries` counts individual conversation entries. `conversations` counts conversations within a project (requires `project::`; currently 1:1 with sessions).

**Examples:**
```bash
# Valid values
target::projects       # Count all projects (default)
target::sessions       # Count sessions in a project
target::entries        # Count entries in a session
target::conversations  # Count conversations in a project

# Invalid values
target::files      # "Invalid target: files"
```

---

### Parameter :: 17. `type::`

Project naming scheme filter for `.list`.

**Type:** [`ProjectType`](types.md#projecttype)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `uuid`, `path`, `all`, `conversation`
- Case-insensitive on input
- Error on invalid: `"Invalid type: {value}. Valid values: uuid, path, all"`
- `conversation` requires `project::` parameter

**Default:** `all`

**Commands:** `.list`
(See [commands.md#command--2-list](commands.md#command--2-list))

**Purpose:** Filters projects by how their directory is named in `~/.claude/projects/`. Path-encoded projects (`-home-user1-pro`) are opened by filesystem path. UUID projects (`8d795a1c-...`) are created by other means. `conversation` lists conversation IDs within a specific project (one per line).

**Examples:**
```bash
# Valid values
type::all           # No filter (default)
type::path          # Path-encoded projects only (e.g., -home-user1-pro)
type::uuid          # UUID-named projects only (e.g., 8d795a1c-...)
type::conversation  # List conversation IDs for a project (requires project::)

# Invalid values
type::both   # "Invalid type: both. Valid values: uuid, path, all"
```

---

### Parameter :: 18. `verbosity::`

Output detail level controlling information density.

**Type:** [`VerbosityLevel`](types.md#verbositylevel)

**Fundamental Type:** Integer wrapper (0-5 range)

**Constraints:**
- Valid range: `0`-`5` inclusive
- Error on out-of-range: `"verbosity must be 0-5, got {value}"`
- Error on non-integer: `"verbosity must be an integer 0-5, got {value}"`

**Default:** `1`

**Alias:** `v`

**Commands:** `.status`, `.list`, `.show`, `.search`, `.projects`
(See individual command sections in [commands.md](commands.md))

**Purpose:** Controls how much information each command outputs. Level `0` is minimal/machine-readable; level `1` is the standard summary; level `2` adds details; level `3` shows all fields; levels `4-5` are reserved.

**Examples:**
```bash
# Valid values
verbosity::0    # Minimal / machine-readable
verbosity::1    # Standard summary (default)
v::2            # Detailed (using alias)
v::3            # Verbose with all fields

# Invalid values
verbosity::6    # "verbosity must be 0-5, got 6"
verbosity::abc  # "verbosity must be an integer 0-5, got abc"
```

**Group:** [Output Control](parameter_groups.md#output-control)

---

### Parameter :: 17. `topic::`

Session topic name appended as a `-{name}` suffix to the base directory path.

**Type:** [`TopicName`](types.md#topicname)

**Fundamental Type:** String (identifier)

**Constraints:**
- Must be non-empty when provided
- Must not contain `/`
- Do NOT include a leading `-` in the value — it is added automatically
- Error on empty: `"topic must be non-empty"`
- Error on slash: `"topic must not contain path separators"`

**Default:** unset (no suffix applied) for `.path`, `.exists`; `default_topic` for `.session.dir`, `.session.ensure`

**Commands:** `.path`, `.exists`, `.session.dir`, `.session.ensure`
(See individual command sections in [commands.md](commands.md))

**Purpose:** Identifies a named session topic within a base directory. Claude Code uses hyphen-prefixed directories (`-default_topic`, `-work`, `-commit`) as session working directories. `topic::` takes the name without the leading hyphen and appends it as `{base}/-{topic}`.

**Examples:**
```bash
# Valid values
topic::default_topic    # → appended as /-default_topic
topic::work             # → appended as /-work
topic::commit           # → appended as /-commit

# Invalid values
topic::                 # "topic must be non-empty"
topic::my/topic         # "topic must not contain path separators"
topic::-default_topic   # (legal — creates /-default_topic... but convention is without leading -)
```

---

### Parameter :: 20. `strategy::`

Resume strategy override for `.session.ensure`.

**Type:** [`StrategyType`](types.md#strategytype)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `resume`, `fresh`
- Case-insensitive on parse
- Error on invalid: `"strategy must be resume|fresh, got {value}"`

**Default:** auto-detect (from conversation history presence)

**Commands:** `.session.ensure`
(See [commands.md#command--13-sessionensure](commands.md#command--13-sessionensure))

**Purpose:** Forces the reported resume strategy instead of auto-detecting it from storage history. When `strategy::resume` is forced and no history exists, the command still reports `resume` (the caller's intent is respected). When `strategy::fresh` is forced and history exists, `fresh` is reported regardless.

**Examples:**
```bash
# Valid values
strategy::resume    # Force resume strategy
strategy::fresh     # Force fresh strategy

# Invalid values
strategy::auto      # "strategy must be resume|fresh, got auto"
strategy::restart   # "strategy must be resume|fresh, got restart"
```

---

### Parameter :: 21. `count::`

Boolean mode flag for `.list` that suppresses the full listing and outputs only the count as a bare integer.

**Type:** Boolean

**Fundamental Type:** Boolean (`0`/`1`, `true`/`false`)

**Constraints:**
- Only meaningful with `type::conversation`
- When `1`: outputs bare integer count + newline, no other output
- When `0` (default): outputs full listing

**Default:** `0` (full listing)

**Commands:** `.list`
(See [commands.md#command--2-list](commands.md#command--2-list))

**Purpose:** Enables scripting use cases where only the count is needed. For example, `clg .list type::conversation count::1 project::abc123` outputs `3` and nothing else.

**Examples:**
```bash
# Count mode on
count::1    # Output bare integer only

# Count mode off (default)
count::0    # Output full listing

# Combined with conversation type
.list type::conversation count::1 project::abc123   # e.g., outputs "3"
```

