# Type System

Semantic newtypes for `claude_storage` CLI parameters. Every parameter uses a named type with validation constraints — never bare primitives.

See [params.md](params.md) for which parameters use each type.

## Type Index

| # | Type | Fundamental | Used By |
|---|------|-------------|---------|
| 1 | [`EntryCount`](#entrycount) | Integer (≥0) | `min_entries::` |
| 2 | [`EntryType`](#entrytype) | String enum | `entry_type::` |
| 3 | [`ExportFormat`](#exportformat) | String enum | `format::` |
| 4 | [`PathSubstring`](#pathsubstring) | String | `path::` in `.list` |
| 5 | [`ProjectId`](#projectid) | String (multi-format) | `project::` |
| 6 | [`ProjectType`](#projecttype) | String enum | `type::` |
| 7 | [`ScopeValue`](#scopevalue) | String enum | `scope::` |
| 8 | [`SessionFilter`](#sessionfilter) | String | `session::` |
| 9 | [`SessionId`](#sessionid) | String | `session_id::` |
| 10 | [`StoragePath`](#storagepath) | String (filesystem) | `path::` (most), `output::` |
| 11 | [`TargetType`](#targettype) | String enum | `target::` |
| 12 | [`TopicName`](#topicname) | String (identifier) | `topic::` |
| 13 | [`VerbosityLevel`](#verbositylevel) | Integer (0-5) | `verbosity::` |
| 14 | [`StrategyType`](#strategytype) | String enum | `strategy::` |

---

### Type :: 1. `EntryCount`

**Purpose:** Non-negative integer representing a minimum session entry threshold. Semantically distinct from general integers — negative values are meaningless for entry counts.

**Fundamental Type:** Wrapper around unsigned integer

**Constants:**
- MIN = 0 (no minimum)
- DEFAULT = unset (no filtering applied)

**Constraints:**
- Range: 0 to i64::MAX
- Negative values rejected: `"min_entries must be ≥ 0, got {value}"`
- Non-integer rejected: `"min_entries must be a non-negative integer, got {value}"`

**Parsing:**
```
Parse string to non-negative integer:
  Input: "0", "10", "100"
  Output: EntryCount(0), EntryCount(10), EntryCount(100)
  Error(negative): "min_entries must be ≥ 0, got {value}"
  Error(non-int): "min_entries must be a non-negative integer, got {value}"

Pseudocode:
  function parse_entry_count(input: string) -> Result<EntryCount>:
    n = parse_int(input)           # error if not integer
    if n < 0:
      return Error("min_entries must be ≥ 0, got " + input)
    return EntryCount(n)
```

**Methods:**
- `get() -> integer` — Raw value accessor
- `is_zero() -> boolean` — True when count is 0 (no minimum)
- `exceeds(actual: integer) -> boolean` — True when actual count is below threshold

**Commands:** `.list`, `.projects`

---

### Type :: 2. `EntryType`

**Purpose:** Semantic type representing which conversation participant authored an entry. Restricts search to user messages, assistant messages, or both.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- USER = `"user"`
- ASSISTANT = `"assistant"`
- ALL = `"all"` (default — no filter)
- DEFAULT = ALL

**Constraints:**
- Valid values: `user`, `assistant`, `all`
- Case-insensitive on parse
- Error on invalid: `"entry_type must be user|assistant|all, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "user", "User", "USER" → EntryType::User
  Input: "assistant" → EntryType::Assistant
  Input: "all" → EntryType::All
  Error: "entry_type must be user|assistant|all, got {value}"

Pseudocode:
  function parse_entry_type(input: string) -> Result<EntryType>:
    match input.to_lowercase():
      "user"      → EntryType::User
      "assistant" → EntryType::Assistant
      "all"       → EntryType::All
      other       → Error("entry_type must be user|assistant|all, got " + input)
```

**Methods:**
- `get() -> string` — Returns canonical lowercase variant name
- `is_all() -> boolean` — True when no filter applied
- `matches(entry: &Entry) -> boolean` — True when entry type matches this filter

**Commands:** `.search`

---

### Type :: 3. `ExportFormat`

**Purpose:** Output serialization format for session export. Determines the structure and encoding of the exported file.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- MARKDOWN = `"markdown"` (default — human-readable)
- JSON = `"json"` (raw entries)
- TEXT = `"text"` (plain transcript)
- DEFAULT = MARKDOWN

**Constraints:**
- Valid values: `markdown`, `json`, `text`
- Case-insensitive on parse
- Error on invalid: `"format must be markdown|json|text, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "markdown" → ExportFormat::Markdown
  Input: "json" → ExportFormat::Json
  Input: "text" → ExportFormat::Text
  Error: "format must be markdown|json|text, got {value}"
```

**Methods:**
- `get() -> string` — Returns canonical lowercase variant name
- `is_default() -> boolean` — True when format is Markdown
- `file_extension() -> string` — Returns `"md"`, `"json"`, or `"txt"`

**Commands:** `.export`

---

### Type :: 4. `PathSubstring`

**Purpose:** Case-insensitive substring matcher against filesystem paths. Semantically distinct from `StoragePath` — this is a filter expression, not a filesystem location.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (no filter applied)

**Constraints:**
- Non-empty string when provided
- Match semantics: case-insensitive substring of the full filesystem path

**Parsing:**
```
Validate non-empty string:
  Input: "willbe" → PathSubstring("willbe")
  Input: ""       → Error("path filter must be non-empty")
```

**Methods:**
- `get() -> string` — Raw substring value
- `matches(path: string) -> boolean` — True if path contains substring (case-insensitive)

**Commands:** `.list` (via `path::`)

**Usage:**
```
.list path::willbe
# Matches: /home/user1/pro/lib/wip_core/willbe/module/core
# Matches: /home/user1/pro/lib/willbe
# Does not match: /home/user1/pro/lib/widgetbe
```

---

### Type :: 5. `ProjectId`

**Purpose:** Multi-format project identifier. Claude Code uses different naming schemes for projects; this type accepts all of them and resolves to the internal path-encoded key.

**Fundamental Type:** Wrapper around string (multi-format)

**Constants:**
- DEFAULT = current working directory (resolved at runtime)

**Accepted Formats:**
- Absolute path: `/home/user1/pro/lib/willbe`
- Path-encoded ID: `-home-user1-pro-lib-willbe`
- UUID: `8d795a1c-c81d-4010-8d29-b4e678272419`
- `Path(...)` form from `.list`: `Path("/home/user1/pro/lib/willbe")`

**Constraints:**
- Non-empty string
- Error if project not found after resolution: `"project not found: {value}"`

**Parsing:**
```
Detect format and resolve to internal key:
  Input starts with "/"  → treat as absolute path, encode to path ID
  Input starts with "-"  → treat as path-encoded ID directly
  Input is UUID pattern  → treat as UUID project name
  Input starts with "Path(" → extract path from Path(...) form, encode
  Input empty            → Error("project must be non-empty")
  Resolve → Error("project not found: " + input) if not in storage
```

**Methods:**
- `get() -> string` — Raw input value
- `path_encoded() -> string` — Resolved path-encoded form
- `is_uuid() -> boolean` — True when project uses UUID naming

**Commands:** `.show`, `.show.project`, `.count`, `.search`, `.export`

---

### Type :: 6. `ProjectType`

**Purpose:** Filter for project naming scheme. Claude Code names projects either by path-encoding their filesystem path or by UUID.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PATH = `"path"` (path-encoded projects)
- UUID = `"uuid"` (UUID-named projects)
- ALL = `"all"` (default — no filter)
- DEFAULT = ALL

**Constraints:**
- Valid values: `uuid`, `path`, `all`
- Case-insensitive on parse
- Error on invalid: `"type must be uuid|path|all, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "path" → ProjectType::Path
  Input: "uuid" → ProjectType::Uuid
  Input: "all"  → ProjectType::All
  Error: "type must be uuid|path|all, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_all() -> boolean` — True when no filter applied
- `matches(project: &Project) -> boolean` — True when project naming matches type

**Commands:** `.list`

---

### Type :: 7. `ScopeValue`

**Purpose:** Controls the discovery boundary for session listing in `.projects`. Mirrors `kbase` scope semantics for consistent cross-tool mental model.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- LOCAL = `"local"` (current project only)
- RELEVANT = `"relevant"` (ancestor chain)
- UNDER = `"under"` (descendant subtree) (default)
- GLOBAL = `"global"` (all projects)
- DEFAULT = UNDER

**Constraints:**
- Valid values: `relevant`, `local`, `under`, `global`
- Case-insensitive on parse
- Error on invalid: `"scope must be relevant|local|under|global, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "local"    → ScopeValue::Local
  Input: "relevant" → ScopeValue::Relevant
  Input: "under"    → ScopeValue::Under
  Input: "global"   → ScopeValue::Global
  Error: "scope must be relevant|local|under|global, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_default() -> boolean` — True when scope is Local
- `requires_path() -> boolean` — True for Under scope (path:: required)
- `ignores_path() -> boolean` — True for Global scope

**Scope comparison:**

| Variant | Direction | Breadth |
|---------|-----------|---------|
| `local` | — | 1 project |
| `relevant` | Up (ancestors) | N projects |
| `under` | Down (descendants) | N projects |
| `global` | — | All projects |

**Commands:** `.projects`

---

### Type :: 8. `SessionFilter`

**Purpose:** Case-insensitive substring matcher against session IDs (JSONL filename stems). Semantically distinct from `SessionId` — this is a pattern for filtering, not a direct identifier.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (no filter applied)

**Constraints:**
- Non-empty string when provided
- Match semantics: case-insensitive substring of session filename stem (without `.jsonl`)

**Parsing:**
```
Validate non-empty string:
  Input: "commit" → SessionFilter("commit")
  Input: ""       → Error("session filter must be non-empty")
```

**Methods:**
- `get() -> string` — Raw substring value
- `matches(session_id: string) -> boolean` — True if session ID contains substring (case-insensitive)

**Commands:** `.list`, `.count`, `.search`, `.projects`

**Usage:**
```
session::commit
# Matches: -commit.jsonl, auto-commit.jsonl, pre-commit.jsonl
# Does not match: -default_topic.jsonl
```

---

### Type :: 9. `SessionId`

**Purpose:** Direct session identifier by filename stem. Used for single-session operations where the exact session is known.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (optional in `.show`; required in `.export`)

**Constraints:**
- Non-empty string
- Error if session not found: `"session not found: {value}"`

**Parsing:**
```
Validate non-empty string, then resolve:
  Input: "-default_topic"                       → SessionId("-default_topic")
  Input: "8d795a1c-c81d-4010-8d29-b4e678272419" → SessionId("8d795a1c-...")
  Input: ""                                     → Error("session_id must be non-empty")
  Resolve → Error("session not found: " + input) if file not in storage
```

**Methods:**
- `get() -> string` — Raw session ID value
- `filename() -> string` — Returns `{id}.jsonl`
- `is_uuid() -> boolean` — True when ID is UUID format
- `is_named() -> boolean` — True when ID is human-readable (starts with `-`)

**Commands:** `.show`, `.export`

---

### Type :: 10. `StoragePath`

**Purpose:** Filesystem path string for storage operations. Accepts both absolute and `~`-prefixed paths.

**Fundamental Type:** Wrapper around string (filesystem path)

**Constants:**
- DEFAULT_ROOT = `~/.claude/` (for `.status`)
- DEFAULT_CWD = current working directory (for `.session`, `.projects`)

**Constraints:**
- Non-empty string
- `~` prefix is shell-expanded to the home directory
- Parent directory must exist for write operations (`output::`)
- Error on empty: `"path must be non-empty"`

**Parsing:**
```
Validate and normalize path:
  Input: "/absolute/path"   → StoragePath("/absolute/path")
  Input: "~/relative/path"  → StoragePath(expand("~/relative/path"))
  Input: "relative/path"    → StoragePath(relative/path)
  Input: ""                 → Error("path must be non-empty")
```

**Methods:**
- `get() -> string` — Raw path string
- `expanded() -> string` — Returns path with `~` expanded
- `exists() -> boolean` — True when path exists on filesystem

**Commands:** `.status`, `.session`, `.projects`, `.export` (via `output::`); `.show.project` (via `project::`)

---

### Type :: 11. `TargetType`

**Purpose:** Selects the granularity for `.count` operations. Determines whether to count projects, sessions within a project, or entries within a session.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PROJECTS = `"projects"` (default)
- SESSIONS = `"sessions"`
- ENTRIES = `"entries"`
- DEFAULT = PROJECTS

**Constraints:**
- Valid values: `projects`, `sessions`, `entries`
- Case-insensitive on parse
- Error on invalid: `"target must be projects|sessions|entries, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "projects" → TargetType::Projects
  Input: "sessions" → TargetType::Sessions
  Input: "entries"  → TargetType::Entries
  Error: "target must be projects|sessions|entries, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_default() -> boolean` — True when target is Projects
- `requires_project() -> boolean` — True for Sessions and Entries
- `requires_session() -> boolean` — True for Entries only

**Commands:** `.count`

---

### Type :: 12. `VerbosityLevel`

**Purpose:** Output detail level controlling information density across all read commands. Provides consistent semantics: `0` is machine-readable, `1` is the standard human-readable default, higher levels add progressively more detail.

**Fundamental Type:** Wrapper around integer (0-5 range)

**Constants:**
- SILENT = 0 (machine-readable / minimal)
- NORMAL = 1 (standard summary — DEFAULT)
- DETAILED = 2 (extended with counts and metadata)
- VERBOSE = 3 (all fields)
- DEFAULT = NORMAL (1)
- MIN = 0
- MAX = 5

**Constraints:**
- Range: 0-5 inclusive
- Error on out-of-range: `"verbosity must be 0-5, got {value}"`
- Error on non-integer: `"verbosity must be an integer 0-5, got {value}"`

**Parsing:**
```
Parse string to integer, validate range:
  Input: "0" → VerbosityLevel(0)
  Input: "1" → VerbosityLevel(1)
  Input: "5" → VerbosityLevel(5)
  Error(out-of-range): "verbosity must be 0-5, got {value}"
  Error(non-int): "verbosity must be an integer 0-5, got {value}"

Pseudocode:
  function parse_verbosity(input: string) -> Result<VerbosityLevel>:
    n = parse_int(input)          # error if not integer
    if n < 0 or n > 5:
      return Error("verbosity must be 0-5, got " + input)
    return VerbosityLevel(n)
```

**Methods:**
- `get() -> integer` — Raw level value
- `is_silent() -> boolean` — True when level is 0
- `is_normal() -> boolean` — True when level is 1 (default)
- `is_detailed() -> boolean` — True when level is 2
- `is_verbose() -> boolean` — True when level ≥ 3
- `default() -> VerbosityLevel` — Returns VerbosityLevel(1)

**Commands:** `.status`, `.list`, `.show`, `.show.project`, `.search`, `.projects`

**Usage:**
```
if verbosity.is_verbose():
  print_all_fields()
elif verbosity.is_detailed():
  print_extended_summary()
elif verbosity.is_normal():
  print_standard_summary()
# silent: print minimal output
```

---

### Type :: 12. `TopicName`

**Purpose:** Session topic identifier appended as `-{name}` to a base directory path. Represents the human-readable name for a Claude Code session working directory.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT_TOPIC = `"default_topic"` (used by `.session.dir` and `.session.ensure` when `topic::` is absent)

**Constraints:**
- Non-empty string when provided
- Must not contain `/` — error: `"topic must not contain path separators"`
- Supplied without leading `-`; the handler prepends `-` when constructing the directory path
- Error on empty: `"topic must be non-empty"`

**Parsing:**
```
Validate non-empty, no slashes:
  Input: "default_topic" → TopicName("default_topic") → dir suffix "-default_topic"
  Input: "work"          → TopicName("work")          → dir suffix "-work"
  Input: ""              → Error("topic must be non-empty")
  Input: "my/topic"      → Error("topic must not contain path separators")
```

**Methods:**
- `get() -> string` — Raw topic name (without leading `-`)
- `as_dir_suffix() -> string` — Returns `-{name}` for appending to base path

**Commands:** `.session`, `.path`, `.exists`, `.session.dir`, `.session.ensure`

---

### Type :: 14. `StrategyType`

**Purpose:** Resume strategy for `.session.ensure`. Determines whether a session should continue an existing conversation (`resume`) or start fresh (`fresh`).

**Fundamental Type:** Wrapper around string enum

**Constants:**
- RESUME = `"resume"` (continue existing conversation)
- FRESH = `"fresh"` (start a new conversation)
- DEFAULT = auto-detect (from conversation history presence)

**Constraints:**
- Valid values: `resume`, `fresh`
- Case-insensitive on parse
- Error on invalid: `"strategy must be resume|fresh, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "resume" → StrategyType::Resume
  Input: "fresh"  → StrategyType::Fresh
  Error: "strategy must be resume|fresh, got {value}"
```

**Auto-detection** (when not forced):
```
if check_continuation(session_dir):
  StrategyType::Resume
else:
  StrategyType::Fresh
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name (`"resume"` or `"fresh"`)
- `is_resume() -> boolean` — True when strategy is Resume
- `is_fresh() -> boolean` — True when strategy is Fresh

**Commands:** `.session.ensure`

