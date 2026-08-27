# claude_storage

CLI tool for exploring and analyzing Claude Code's filesystem-based conversation storage.

## Files

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest: deps, features, build script |
| `build.rs` | Transforms YAML command definitions to static PHF registry |
| `unilang.commands.yaml` | Command definitions (16 commands) |
| `src/` | CLI pipeline, command routines, binary entry points |
| `tests/` | Integration and parameter validation tests |
| `docs/` | Behavioral requirements: features, CLI reference, operation docs |
| `examples/` | Usage examples for storage API |
| `changelog.md` | Notable changes by version |
| `license` | MIT license text (`license.workspace = true` in `Cargo.toml`) |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

This crate provides a command-line interface for querying Claude Code's conversation storage at `~/.claude/`. It wraps the `claude_storage_core` library with an interactive REPL and one-shot command interface.

**v1.0 Status**: Core library (`claude_storage_core`) is production-ready with comprehensive validation (188 tests, production session parsing). CLI wrapper commands `.status`, `.list`, and `.count` are fully validated. For programmatic access or advanced usage, we recommend using the `claude_storage_core` library API directly (see "library api" section below).

**Extraction context**: This is the CLI-focused crate after extracting core library functionality to `claude_storage_core` (2025-11-29).

## installation

```bash
cargo install --path . --features cli
```

Or run directly:
```bash
cargo run --features cli
```

## usage

### repl mode (interactive)

```bash
cargo run --features cli
```

```text
claude_storage> .status
Storage: "/home/user/.claude"
Projects: 230 (UUID: 14, Path: 216)
Sessions: 7546 (Main: 1061, Agent: 6485)
Entries: 323231

claude_storage> .list target::projects
UUID projects: 14
Path projects: 216

claude_storage> .count target::sessions
Total sessions: 7546

claude_storage> exit
```

### one-shot mode (scripting)

```bash
# Get storage statistics
cargo run --features cli -- .status

# Count projects
cargo run --features cli -- .count target::projects

# List projects with filtering
cargo run --features cli -- .list target::projects filter::path

# Show session details
cargo run --features cli -- .show session_id::abc123
```

## commands

### .status

Show storage statistics (projects, sessions, entries, tokens).

**Parameters**:
- `path::{value}` (optional, default: `~/.claude/`) - Custom storage path
- `show_tokens::1` (optional, default: 0) - Show token usage statistics (triggers full JSONL parse — slow for large storage)

**Example**:
```bash
.status show_tokens::1
```

### .list

List projects or sessions with optional filtering.

**Deprecated**: use `.projects` instead — `detail::projects` (project-only view), `filter::` (path substring), and `ids::`/`count::` (conversation-ID scripting) cover `.list`'s former capabilities.

**Parameters**:
- `type::{uuid|path|all}` (optional, default: all) - Filter by project type
- `show_sessions::{0|1}` (optional, default: 0) - Show sessions for each project (auto-enabled when session filters provided, explicit 0/1 overrides)
- `path::{value}` (optional) - Filter projects by path (supports smart resolution, see below)
- `agent::{0|1}` (optional) - Filter sessions by type (auto-enables session display)
- `min_entries::N` (optional) - Filter sessions by minimum entry count (auto-enables session display)
- `session::{substring}` (optional) - Filter sessions by ID substring (auto-enables session display)
- `project::{id}` (optional) - Project ID (required for `type::conversation`)
- `count::1` (optional, default: 0) - Output only the count as a bare integer instead of the full list
- `scope::{value}` (optional, default: global) - Discovery boundary for project listing when `type::` is `all`

**Path Parameter - Smart Resolution**:

The `path::` parameter supports both shell-style path resolution and pattern matching:

- **Special paths** (resolved to absolute paths):
  - `path::.` → Current working directory
  - `path::..` → Parent directory
  - `path::~` → Home directory
  - `path::~/subdir` → Home directory + relative path

- **Patterns** (substring matching):
  - `path::assistant` → Match any path containing "assistant"
  - `path::storage` → Match any path containing "storage"

**Examples**:
```bash
# List all projects
.list

# List path-based projects only
.list type::path

# Path resolution (current directory)
cd /home/user/project
.list path::.

# Path resolution (parent directory)
.list path::..

# Path resolution (home directory)
.list path::~

# Pattern matching (backward compatible)
.list path::assistant

# Filter sessions with auto-enable
.list session::commit          # Auto-enables session display
.list agent::1 min_entries::10 # Agent sessions with 10+ entries

# Combine filters
.list path::claude_storage session::default
```

### .show

Display session or project details with **conversation content by default** (REQ-011: Content-First Display).

**Smart Behavior** (adapts to parameters):
- **No parameters** → Shows current directory project (all sessions)
- **session_id only** → Shows that session in current project with conversation content
- **project only** → Shows that project (all sessions)
- **Both parameters** → Shows that session in that project with conversation content

**Parameters**:
- `session_id::{uuid-or-agent-id}` (optional) - Session UUID or agent-{hex}
- `project::{path-or-id}` (optional) - Project path or UUID (default: current directory)
- `show_entries::1` (optional) - Show all entries (backward compat with old UUID list format); only has an effect combined with `show_metadata::1`
- `show_metadata::1` (optional) - Show metadata only (old behavior, no conversation content)
- `show_stat::1` (optional) - Accepted for backward compatibility; has no effect (content mode already shows entry counts and timestamps unconditionally)
- `show_tokens::1` (optional) - Show token usage section
- `detail::{value}` (optional) - Project-overview verbosity: summary only, or also list every session
- `last::{n}` (optional) - Trailing messages from the most-recently-active session (0 = all)
- `index::{n}` (optional) - 1-based position narrowing the in-scope message set to exactly one message
- `fields::{list}` (optional) - Attribute projection: comma-separated names from the 18 canonical fields, or `all`; switches per-entry rendering to an explicit field block
- `scope::{value}` (optional, default: local) - Project search boundary when `session_id::` is given without `project::`
- `path::{value}` (optional, default: current directory) - Base path for scope resolution when `session_id::` is given without `project::`

**Default Behavior** (NEW):
Shows actual conversation content in readable chat-log format. No parameters needed to read messages.

**Examples**:
```bash
# Show current directory project (all sessions)
cd /home/user/project
.show

# Show session with conversation content (default)
.show session_id::abc123

# Show session in different project
.show session_id::abc123 project::/home/user/project

# Metadata only (old behavior)
.show session_id::abc123 show_metadata::1

# Show token usage alongside metadata
.show session_id::abc123 show_metadata::1 show_tokens::1
```

**Content Format**:
```text
Session: feed0002... (2893 entries)
Path: /home/user/.claude/projects/-home-user-project/feed0002-....jsonl
Agent Session: false
Total Entries: 2893
User Entries: 1447
Assistant Entries: 1446
First Entry: 2025-12-01T08:15:23.000Z
Last Entry: 2025-12-02T09:57:00.000Z

[2025-12-02 09:57] User:
last 3 biig tasks solved in this context?

[2025-12-02 09:57] Assistant:
I'll analyze the recent conversation history...

**Recent Major Tasks Completed:**
1. **tree_fmt Standardization**
2. **Path Filter Bug Investigation**
3. **Test Suite Fixes**
```

### .count

Fast counting operations (projects, sessions, entries).

**Parameters**:
- `target::projects|sessions|entries` (required)
- `project::{id}` (for sessions/entries)
- `session::{id}` (for entries)
- `scope::{value}` (optional, default: global) - Boundary for what gets counted under `target::projects` or `target::sessions` without `project::`
- `path::{value}` (optional, default: `~/.claude/`) - Custom storage root path

**Examples**:
```bash
.count target::projects
.count target::sessions project::-home-user-pro
.count target::entries session::abc123
```

### .search

Search session content for query string.

**Parameters**:
- `query::{text}` (required) - Search query (case-insensitive by default)
- `project::{id}` (optional) - Limit search to specific project
- `session::{id}` (optional) - Limit search to specific session
- `case_sensitive::1` (optional) - Enable case-sensitive matching
- `entry_type::user|assistant` (optional) - Filter by entry type
- `scope::{value}` (optional, default: global) - Project search boundary when `project::` is not given
- `path::{value}` (optional, default: current directory) - Base path for scope resolution when `project::` is not given

**Examples**:
```bash
.search query::error
.search query::"session management" case_sensitive::1
.search query::implement project::-home-user-pro
```

### .export

Export session to file (markdown, JSON, or text).

**Parameters**:
- `session_id::{id}` (required) - Session ID to export
- `output::{path}` (required) - Output file path
- `format::markdown|json|text` (optional, default: markdown) - Export format
- `project::{id}` (optional) - Project ID if not in current directory
- `scope::{value}` (optional, default: local) - Project search boundary for source session lookup when `project::` is not given
- `path::{value}` (optional, default: current directory) - Base path for scope resolution when `project::` is not given

**Formats**:
- **markdown** (.md) - Human-readable with metadata and formatted entries
- **json** (.json) - Machine-readable structured format
- **text** (.txt) - Simple conversation transcript

**Examples**:
```bash
.export session_id::-default_topic output::conversation.md
.export session_id::abc123 format::json output::session.json
.export session_id::xyz789 format::text output::transcript.txt project::-home-user-pro
```

**Note**: Sessions may contain non-conversation metadata entries (queue-operation, summary) which are automatically skipped during export. Only conversation entries (user/assistant messages) are included in the exported output.

### .projects

Scoped project list with per-project session aggregation.

**Parameters**:
- `scope::local|relevant|under|global|around` (optional, default: around) - Project discovery scope
- `path::{value}` (optional, default: current directory) - Base path for scope resolution
- `filter::{text}` (optional) - Filter resolved projects by decoded path substring (case-insensitive)
- `type::uuid|path|all` (optional) - Project naming filter
- `detail::projects|sessions` (optional, default: projects) - One line per project, or every session listed
- `session::{id}` (optional) - Filter sessions by ID substring (case-insensitive)
- `agent::0|1` (optional) - Session type filter (0 = main only, 1 = agent only, unset = all)
- `min_entries::{n}` (optional) - Filter by minimum entry count
- `since_days::{n}` (optional) - Only sessions modified within the last N days (0 = last 24 hours)
- `limit::{n}` (optional, default: 0) - Max main sessions per project (0 = unlimited)
- `show_tree::1` (optional) - Nest projects by directory, or agents under their root session
- `show_topic::1` (optional) - Append each conversation's first user message to its line
- `live::0|1` (optional) - Filter by attached Claude Code process (unset = all)
- `ids::1` (optional) - Output raw conversation IDs for `project::` (scripting mode)
- `project::{id}` (optional) - Project ID; scopes `ids::` output (required with `ids::1`)
- `count::1` (optional) - With `ids::1`, output only the count as a bare integer

**Examples**:
```bash
.projects                                   # Around the current directory
.projects scope::global show_tree::1        # Every project, nested by directory
.projects detail::sessions since_days::7    # Sessions touched in the last week
.projects scope::global live::1             # Only projects with a process attached
.projects ids::1 project::-home-user-pro    # Conversation IDs, one per line
```

**Note**: `live::` infers attachment from the process table and `history.jsonl` — it
can only report positives. A blank `STATUS` column means nothing was detected, which
is not the same as nothing running. See `docs/algorithm/002_session_liveness.md`.

### .project.path

Compute the Claude storage path for a project directory. Pure computation — the
path need not exist.

**Parameters**:
- `path::{dir}` (optional, default: current directory) - Directory to compute the storage path for
- `topic::{name}` (optional) - Session topic name, appended as `-{topic}`

**Examples**:
```bash
.project.path
.project.path path::/home/user/pro/app
.project.path topic::review
```

### .project.exists

Check whether a project directory has existing conversation history.

**Parameters**:
- `path::{dir}` (optional, default: current directory) - Directory to check
- `topic::{name}` (optional) - Session topic name, appended as `-{topic}`

**Exit codes**: `0` = history exists, `1` = it does not.

**Examples**:
```bash
.project.exists
if clg .project.exists path::/home/user/pro/app; then echo "has history"; fi
```

### .session.dir

Compute the session working directory path. Does not create it.

**Parameters**:
- `path::{dir}` (optional, default: current directory) - Project directory
- `topic::{name}` (optional, default: default_topic) - Session topic name

**Examples**:
```bash
.session.dir
.session.dir path::/home/user/pro/app topic::review
```

### .session.ensure

Ensure the session directory exists and report whether to resume or start fresh.

**Parameters**:
- `path::{dir}` (optional, default: current directory) - Project directory
- `topic::{name}` (optional, default: default_topic) - Session topic name
- `strategy::resume|fresh` (optional) - Force a strategy instead of inferring one

**Examples**:
```bash
.session.ensure
.session.ensure topic::review strategy::fresh
```

### .session.path

Print the absolute session `.jsonl` file path for a directory.

**Parameters**:
- `path::{dir}` (optional, default: current directory) - Project directory the session belongs to
- `latest::1` (optional) - Most recent qualifying session — the default selector
- `session::{uuid}` (optional) - Explicit session UUID; pure computation, the file need not exist
- `topic::{name}` (optional) - Fork-mode topic name resolved via the shared UUIDv5 rule

`latest::`, `session::`, and `topic::` are mutually exclusive.

**Exit codes**: `2` when `latest::` is selected and the storage has no qualifying session.

**Note**: `topic::` here means something different from every other command's
`topic::`. Elsewhere it is the `-{topic}` directory suffix; here it is a fork-mode
name hashed through UUIDv5 against the canonical physical directory, byte-identical
to `clr topics --file NAME`.

**Examples**:
```bash
.session.path
.session.path session::bff63952-8a23-4794-ad56-3a8e4fc4e9a9
.session.path topic::review
```

### .tail

Print the last N conversation turns of the current directory's session.

**Parameters**:
- `last::{n}` (optional, default: 4) - Number of trailing turns (0 = all)
- `full::1` (optional) - Print every body line instead of folding long turns after 8 lines
- `compact::1` (optional) - One line per turn: ordinal, age, speaker, elided first line
- `path::{dir}` (optional, default: current directory) - Directory to resolve the project from
- `topic::{name}` (optional) - Session topic name; unset falls back to the most recently modified session

**Examples**:
```bash
.tail
.tail last::20 full::1
.tail compact::1 last::0
```

### .usage

Per-session usage table: turns, token totals, wall-clock duration, and working
directory, most recent first.

**Parameters**:
- `scope::local|relevant|under|global|around` (optional, default: local) - Project selection scope
- `path::{dir}` (optional, default: current directory) - Anchor directory for scope resolution
- `depth::{n}` (optional, default: 3) - Component-distance cap for under/relevant/around (0 = unbounded)
- `limit::{n}` (optional, default: 0) - Max rows across the whole result set (0 = all)

**Examples**:
```bash
.usage
.usage scope::global limit::20
.usage scope::under depth::0
```

### .rollup

Token-usage rollup: group by session, project, model, or day; filter by model;
sort by any column; project only the columns you want.

**Parameters**:
- `group::session|project|model|day` (optional, default: session) - Grouping dimension
- `sort::total|input|output|cache|max_context|calls|sessions|group` (optional, default: total) - Sort column
- `order::asc|desc` (optional, default: desc) - Sort direction
- `model::{text}` (optional) - Model substring filter, applied before grouping
- `columns::{list}` (optional) - Comma-separated projection from `group,sessions,calls,input,output,cache,max_context,total,percent,first,last`
- `scope::local|relevant|under|global|around` (optional, default: local) - Project selection scope
- `path::{dir}` (optional, default: current directory) - Anchor directory for scope resolution
- `depth::{n}` (optional, default: 3) - Component-distance cap for under/relevant/around (0 = unbounded)
- `limit::{n}` (optional, default: 0) - Max rows after sorting (0 = all)

**Examples**:
```bash
.rollup
.rollup group::model sort::total order::desc
.rollup group::day scope::global limit::30
.rollup group::project columns::group,sessions,total,percent
```

### .cost

Per-conversation cost table: exact token counts, cache read/write split,
compactions, max context, and estimated USD cost.

**Parameters**:
- `session_ids::{list}` (optional) - Comma-separated session IDs or unique ID prefixes, searched across all projects; defaults to the most recent session of the current directory's project
- `path::{dir}` (optional, default: current directory) - Directory whose project anchors default session resolution
- `agents::0|1` (optional, default: 1) - Fold agent (subagent) sessions into each conversation's row

**Examples**:
```bash
.cost
.cost session_ids::bff63952,98da5af5
.cost agents::0
```

## scripting integration

**Exit codes**:
- 0: Success
- 1: Error

**Examples**:
```bash
# Get project count
PROJECT_COUNT=$(clg .count target::projects | grep -oP '\d+')

# Check if session exists
if clg .show session_id::abc123 &>/dev/null; then
  echo "Session exists"
fi

# Export statistics
clg .status show_tokens::1 > storage_stats.txt

# Conversation IDs for a project, one per line
clg .projects ids::1 project::-home-user-pro

# Absolute path of the latest session file
SESSION_FILE=$(clg .session.path)
```

## library api

For programmatic access to Claude Code storage, use `claude_storage_core` directly:

```toml
[dependencies]
claude_storage_core = "1.5.1"
# Or for local development:
# claude_storage_core = { path = "../claude_storage_core" }
```

```rust,no_run
use claude_storage_core::{ Storage, ProjectId };

fn main() -> claude_storage_core::Result< () >
{
  let storage = Storage::new()?;
  for project in storage.list_projects()?
  {
    println!( "Project: {:?}", project.id() );
  }
  Ok( () )
}
```

## architecture

**Dependencies**:
- `claude_storage_core` - Core library for all storage operations
- `unilang` - CLI framework for command parsing
- `phf` - Perfect hash functions for static command registry

**Build system**:
- `build.rs` - Transforms YAML command definitions to static PHF registry
- `unilang.commands.yaml` - Command definitions (16 commands)
- Generated code: Static command map with O(1) lookup

**Command routines** (`src/cli/`):
- `status_routine` - Global statistics aggregation
- `list_routine` - Filtered listing
- `show_routine` - Session detail display
- `count_routine` - Fast counting
- `search_routine` - Content search
- `export_routine` - Session export
- `projects_routine` - Project discovery and listing
- `project_path_routine` - Resolve project ID to filesystem path
- `project_exists_routine` - Check project existence
- `session_dir_routine` - Resolve session working directory
- `session_ensure_routine` - Create session working directory
- `session_path_routine` - Resolve session ID to filesystem path
- `tail_routine` - Live-tail session content
- `usage_routine` - Token usage aggregation
- `rollup_routine` - Cross-session rollup summary
- `cost_routine` - Cost estimation from token usage

## documentation

- **Documentation**: `docs/` - Behavioral requirements, CLI reference, feature docs
- **Format docs**: `docs/` - Storage organization, file formats, advanced topics

## testing

**Container tests**: Run via `./verb/test` from the crate directory.

**Core library tests**: 188 tests in `claude_storage_core` crate
- Entry parsing and validation
- Path encoding/decoding
- JSON parser
- Filtering system
- Content search
- Export functionality (markdown, JSON, text)
- Statistics aggregation
- Bug reproducers with comprehensive documentation

**CLI tests**: 1025 tests across 99 integration test files
- Storage operations tests (global stats, project listing)
- Session operations tests (show, stats, entry counts)
- Counting operations tests (projects, sessions, entries)
- Full workflow integration test
- CLI sanity tests (build, features)

**Targeted run**: `./verb/test_only <name_substring>` from the crate directory.

## license

MIT
