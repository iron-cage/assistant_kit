# Command :: 7. `.projects`

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
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `around` | Session discovery scope |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Base path for scope resolution |
| `session::` | [`SessionFilter`](../type/08_session_filter.md) | optional | — | Filter sessions by ID substring |
| `agent::` | Boolean | optional | — | Session type filter (`0`=main, `1`=agent) |
| `min_entries::` | [`EntryCount`](../type/01_entry_count.md) | optional | — | Minimum entry count threshold |
| `limit::` | Integer | optional | `0` | Max main sessions per project at v1 (`0` = unlimited) |
| `verbosity::` | [`VerbosityLevel`](../type/12_verbosity_level.md) | optional | `1` | Output detail level |

`scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md). Session filters belong to [Session Filter](../param_group/04_session_filter.md).

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
claude_storage .projects scope::under path::/home/alice/projects

# All sessions, agent only, with entries
claude_storage .projects scope::global agent::1 min_entries::50

# Show at most 5 sessions per project
claude_storage .projects scope::global limit::5
```

**Notes:**
- `scope::relevant` walks UP from cwd to `/`, collecting sessions from every project at each ancestor level
- Distinct from `.project.exists`: that checks existence (exit 0/1); this lists conversations
- **Fixed (issue-024)**: `scope::local/relevant/under` previously returned 0 results when the base path contained underscores (e.g., `my_project`). Root cause: lossy encoding mapped `_` and `/` identically; decoded paths diverged from real paths. Fixed by comparing encoded paths directly against raw storage directory names.
- **Fixed (issue-029)**: `scope::under` (and all scopes at verbosity ≥ 1) previously displayed project path headers with underscore-named directories split as path separators (e.g., `my_project` → `my/project`). Root cause: `decode_project_display` heuristic defaulted to `/` for every `-` boundary; underscore-named dirs were indistinguishable from path separators in the encoded form. Fixed by adding a filesystem-guided fallback that walks the real directory tree to resolve ambiguous boundaries.
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

Family display: agents are grouped by parent session into families. Each root session line shows an inline `[N agents: breakdown]` suffix. Roots with no agents show no bracket suffix. Orphan families (root deleted) use `?` marker. When `agent::` filter is set, family grouping is disabled — flat display.

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

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | — |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
