# Command :: 7. `.projects`

### Scope

- **Purpose**: Specify the `.projects` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.projects`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Project list with scope control; conversations are grouped by project directory and one entry is shown per project (not per session file). Bare invocation shows all projects in the bidirectional neighborhood (ancestors + current + descendants via `scope::around`) as a terse one-line-per-project overview.

**What a "project" is here:** one directory under the storage root, created per distinct working directory `claude` was launched from — a *cwd bucket*, not a repository, crate, or workspace. One repo entered from five subdirectories produces five projects; a scratch directory used once produces one. This is why the tree layout (`show_tree::1`) is useful: it re-assembles those buckets into the directory structure they actually came from.

**Absorbed from `.list` (deprecated, see [`02_list.md`](02_list.md)):** `.projects` is now the single command for both project-only and session-detail views. `detail::` selects the view (`projects` = terse overview, default; `sessions` = full detail); `filter::` provides the path-substring narrowing `.list`'s `path::` used to; `ids::` (paired with `count::`) provides the raw conversation-ID scripting shortcut `.list type::conversation` used to; `type::` (narrowed to `uuid`/`path`/`all`) filters by project naming scheme. This is a deliberate consolidation, not a Representation Absorption Test merge — see `../command_group/readme.md § Command Removal: .list -> .projects` for why the two commands never qualified as an automatic merge and why they were consolidated anyway.

**Parameters:** `scope::`, `path::`, `filter::`, `type::`, `detail::`, `session::`, `agent::`, `min_entries::`, `ids::`, `project::`, `count::`, `limit::`, `show_tree::`, `since_days::`, `show_topic::`, `live::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .projects
claude_storage .projects scope::around
claude_storage .projects scope::relevant
claude_storage .projects scope::under path::PATH
claude_storage .projects scope::global [agent::1] [min_entries::N]
claude_storage .projects limit::5
claude_storage .projects scope::global since_days::20 show_topic::1
claude_storage .projects scope::global detail::sessions
claude_storage .projects scope::global show_tree::1
claude_storage .projects scope::global filter::SUBSTR
claude_storage .projects scope::global type::uuid
claude_storage .projects project::PROJECT ids::1
claude_storage .projects project::PROJECT ids::1 count::1
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `around` | Session discovery scope |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Base path for scope resolution |
| `filter::` | [`PathSubstring`](../type/04_path_substring.md) | optional | — | Filter resolved projects by path substring |
| `type::` | [`ProjectType`](../type/06_project_type.md) | optional | `all` | Project naming filter (`uuid`, `path`, `all`) |
| `detail::` | `DetailLevel` (new type — `type/`) | optional | `projects` | Output detail: `projects` (terse overview, default) or `sessions` (every session listed) |
| `session::` | [`SessionFilter`](../type/08_session_filter.md) | optional | — | Filter sessions by ID substring |
| `agent::` | Boolean | optional | — | Session type filter (`0`=main, `1`=agent) |
| `min_entries::` | [`EntryCount`](../type/01_entry_count.md) | optional | — | Minimum entry count threshold |
| `ids::` | Boolean | optional | `0` | Output raw conversation IDs for `project::` (scripting mode) |
| `project::` | String | required with `ids::1` | — | Project ID; scopes `ids::` output |
| `count::` | Boolean | optional | `0` | With `ids::1`: output only the count as a bare integer |
| `limit::` | Integer | optional | `0` | Max main sessions per project (`0` = unlimited) |
| `show_tree::` | Boolean | optional | `0` | Tree layout: projects nested by directory (`detail::projects`) or agents under roots (`detail::sessions`) |
| `since_days::` | Integer | optional | — | Only sessions modified within the last N days (`0` = last 24 hours) |
| `show_topic::` | Boolean | optional | `0` | Append first user message text to session lines |
| [`live::`](../param/44_live.md) | Boolean | optional | unset | Keep only projects with (`1`) or without (`0`) an attached Claude Code process |

`scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md). Session filters belong to [Session Filter](../param_group/04_session_filter.md). `show_tree::` and `show_topic::` belong to [Output Control](../param_group/01_output_control.md). `project::` belongs to [Project Scope](../param_group/02_project_scope.md). `filter::`, `type::`, `detail::`, `ids::`, and `count::` are standalone `.projects`-only parameters with no group (see [`../param/30_detail.md`](../param/30_detail.md) for why `detail::` specifically doesn't join Output Control despite looking like a display-shaping toggle).

**Algorithm (6 steps):**
1. Early dispatch — if `ids::1`: require `project::`, load project, build session families, group into conversations, output conversation IDs (or bare count when `count::1`); ported from `.list`'s former `type::conversation` path, plus the one filter that has to be re-applied here because this branch answers before step 6 ever runs — `live::` as a predicate on the named project, probed only when set (→ [`../param/44_live.md`](../param/44_live.md) § With `ids::1`)
2. Parse scope, filters, and resolve base path — validate `scope::`, `type::`, `min_entries::` non-negative, `limit::` non-negative, `since_days::` non-negative; encode base path for scope comparison
3. Filter projects by scope predicate — `local`: exact match + topic variants; `relevant`: ancestor chain (component-wise); `under`: subtree (component-wise); `around`: union of under + relevant; `global`: all projects — then narrow by `type::` naming scheme (`uuid`/`path`/`all`) and `filter::` path substring
4. Collect sessions per matching project — apply session filter (agent, min_entries, session ID substring), then the `since_days::` mtime window (cutoff `now - max(N,1) × 24h`; unreadable mtime = excluded); group by decoded display path (filesystem-guided decode resolves `_` vs `/` ambiguity)
5. Sort and aggregate — sessions by mtime descending within each project; projects by most-recent session mtime descending; exclude zero-byte placeholder sessions
6. Format output per `detail::` — `detail::projects` (default): terse overview, one row per project under a totals summary line, laid out as a flat recency table or (`show_tree::1`) a directory tree; `detail::sessions`: family display (agents grouped under root sessions with `[N agents: breakdown]` brackets) or tree display (`show_tree::1`: `├─`/`└─` connectors); with `show_topic::1`, append each session's first user message (flattened, max 90 chars); apply `limit::` cap with `... and N more` hint

**Default invocation:**

Bare `clg .projects` uses `scope::around` and `detail::projects` — showing all projects in the bidirectional neighborhood of cwd (ancestors upward to `/` plus all descendants), one line each. No sessions in scope → `No active project found.`

The default was `detail::sessions` while `.list` was being absorbed, purely to keep bare `.projects` behaving as it had before the merge. That rationale expired with `.list`: a command named `.projects` should answer "which projects?", and on a machine with hundreds of cwd buckets the session-detail default expands that answer into thousands of lines. Pass `detail::sessions` to get the old view.

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

# Conversations active in the last 20 days, with their opening topic
claude_storage .projects scope::global since_days::20 show_topic::1

# Every project on the machine, one line each
claude_storage .projects scope::global

# Same rows, nested by directory
claude_storage .projects scope::global show_tree::1

# Full session detail beneath each project — the pre-flip default
claude_storage .projects scope::global detail::sessions

# Filter projects by path substring — replaces `.list path::SUBSTR`
claude_storage .projects scope::global filter::assistant

# List only UUID-identified projects — replaces `.list type::uuid`
claude_storage .projects scope::global type::uuid

# List conversation IDs for a specific project (scripting) — replaces `.list type::conversation`
claude_storage .projects project::abc123 ids::1

# Count conversations in a project as a bare integer — replaces `.list type::conversation count::1`
claude_storage .projects project::abc123 ids::1 count::1
```

**Notes:**
- `scope::relevant` walks UP from cwd to `/`, collecting sessions from every project at each ancestor level
- Distinct from `.project.exists`: that checks existence (exit 0/1); this lists conversations
- `session::`, `agent::`, and `min_entries::` never auto-enable session lines — under the default `detail::projects` they narrow the per-project counts; pass `detail::sessions` to see individual sessions
- `type::uuid`/`type::path` narrow which projects are considered, independent of `scope::`'s discovery boundary — the two compose (scope resolves candidates, `type::` and `filter::` narrow them further)
- `ids::1` requires `project::` and lists one conversation ID per line; `count::1` with `ids::1` outputs only the count as a bare integer (useful for scripting)
- `limit::` and `show_topic::` only affect rendering under `detail::sessions` — they are no-ops under `detail::projects` (no session lines to cap or annotate). `show_tree::` applies to both, selecting the tree layout at each level: projects nested by directory under `detail::projects`, agents nested under root sessions under `detail::sessions`
- `live::` filters projects, never sessions: narrowing the session set here would desynchronize every per-project count from what renders (the issue-034 class of defect). Under `detail::sessions` the driven conversation is *marked*, and its siblings stay listed for context
- **Fixed (issue-024)**: `scope::local/relevant/under` previously returned 0 results when the base path contained underscores (e.g., `my_project`). Root cause: lossy encoding mapped `_` and `/` identically; decoded paths diverged from real paths. Fixed by comparing encoded paths directly against raw storage directory names.
- **Fixed (issue-029)**: `scope::under` (and all scopes) previously displayed project path headers with underscore-named directories split as path separators (e.g., `my_project` → `my/project`). Root cause: `decode_project_display` heuristic defaulted to `/` for every `-` boundary; underscore-named dirs were indistinguishable from path separators in the encoded form. Fixed by adding a filesystem-guided fallback that walks the real directory tree to resolve ambiguous boundaries.
- **Fixed (issue-030)**: Session path headers previously showed only the base directory, truncating hyphen-prefixed topic components (e.g., `src/-default_topic` was shown as `src`). Root cause: `decode_project_display` stripped all `--topic` suffixes before decoding. Fixed by decoding the base path with filesystem guidance (resolves `_` vs `/` ambiguity per issue-029), then appending topic components as hyphen-prefixed directory names. **Display-path invariant**: topic components must always be appended regardless of whether the directory currently exists on disk — the storage key encodes the actual CWD at session time and must be decoded as-is.
- **Fixed (issue-035)**: The issue-030 fix introduced an incorrect filesystem existence check — topic components were only appended when `candidate.exists()` was true. Sessions recorded in `dir/-commit` displayed as `dir` after the `-commit` directory was deleted, obscuring which working directory the session used. Root cause: `decode_project_display` called `candidate.exists()` and broke at the first missing topic dir. Fixed by removing the existence guard from the topic-extension loop; all topic components are always appended unconditionally — filesystem state at query time must not affect which CWD a session is attributed to. (Task 025.)
- **Fixed (issue-031)**: `scope::under` previously included sessions from sibling modules whose names start with the base name followed by `_` (e.g., `claude_storage_core` matched when base was `claude_storage`). Root cause: `encode_path` maps both `_` and `/` to `-`, so string `starts_with` cannot distinguish a child path from an underscore-suffixed sibling — both produce the same encoded prefix. Fixed by a two-stage predicate: string prefix is fast-reject only; `decode_path_via_fs` + `Path::starts_with` (component-wise) provides correct disambiguation.
- **Fixed (issue-032)**: `scope::relevant` previously included sessions from sibling projects whose encoded name is a string prefix of the current path's encoded form (e.g., `/base` matched when current path was `/base_extra`). Root cause: `is_relevant_encoded` used `encoded_base.starts_with(dir_name + "-")` which cannot distinguish a true ancestor (`base/sub`) from a same-level sibling with an underscore suffix (`base_extra`). Fixed by the same two-stage predicate as issue-031: `decode_path_via_fs` + `base_path.starts_with(decoded_path)` (component-wise) for disambiguation.

**Output format:**

`detail::projects` (default) renders a totals summary line followed by one row per project. The default layout is a flat table sorted by recency descending:

```
5 projects · 6 conversations · 19 agents · 2 live (1 working, 1 waiting)

  LAST      CONV  AGENTS  STATUS             PROJECT
▸ 2h ago  2 conv   12 ag  ● working          ~/pro/lib/assistant_kit/claude_storage
  5h ago  1 conv    3 ag  ○ waiting          ~/pro/lib/assistant_kit
  1d ago  1 conv       ·                     ~/pro/lib/assistant_kit/module/claude_storage/docs
  3d ago  1 conv    4 ag                     ~/pro/lib/wtools
  6d ago  1 conv       ·             ⚠ gone  ~/pro/lib/assistant/kit/-commit
```

Column widths adapt to content, and the `STATUS` and `⚠ gone` columns are each reserved only when at least one row needs it.

Column semantics:

| Column | Meaning |
|--------|---------|
| gutter | `▸` marks the project matching the current working directory |
| `LAST` | Relative age of the most recent non-empty session |
| `CONV` | Root conversations in the project |
| `AGENTS` | Agent sessions across those conversations; `·` when zero |
| `STATUS` | A Claude Code process is attached — see below |
| `⚠ gone` | The decoded path names a directory that no longer exists — see below |
| `PROJECT` | Full decoded path, always printed in full (never factored against a shared prefix, so every row stays copyable into `cd`, `grep`, or `project::`) |

The `⚠ gone` marker exists because path encoding is lossy: `/`, `_`, and `.` all collapse to `-`, so a decoded path is only trustworthy while the directory it names still exists to disambiguate it. Once deleted — typically a `-commit` scratch directory — the rendered path is a reconstruction, and is labelled as one rather than presented as fact. The path is still shown; it is the only identifier the storage key carries.

### Session Liveness

`LAST` answers "when was this last written"; `STATUS` answers the orthogonal question "is anything running against it right now". They are genuinely different facts, not two views of one — an attached session sitting at an idle prompt routinely goes quiet for the better part of an hour, while a session that exited thirty seconds ago still sorts to the top. Full inference rules and their measured basis: [`../../algorithm/002_session_liveness.md`](../../algorithm/002_session_liveness.md).

| Cell | Meaning | What it is telling you |
|------|---------|------------------------|
| `● working` | Process attached, wrote within the last 60 s | Mid-turn — output is still arriving |
| `○ waiting` | Process attached, quiet longer than that | A terminal is open on this and is waiting for you to type |
| *(blank)* | No process attached | Nothing running; resume it with `claude -c` if you want it back |

The summary line gains a `· N live (X working, Y waiting)` clause on the same condition. When every live row shares one state the breakdown collapses to that state's name — `· 3 live (waiting)` — for the same reason the `agents` clause disappears at zero: a half that can only ever read `0` is width spent to say nothing.

Two constraints follow from liveness being *inferred* rather than recorded — Claude Code writes no lock file, keeps no handle on the session JSONL, and carries no session id in its environment:

- **The column never renders a negative.** It appears only when at least one attached process was actually found. Detection reads the process table, so inside a container, or on a platform without `/proc`, nothing is visible even while sessions are running — an absent `STATUS` column means "nothing detected", never "nothing live". [`live::1`](../param/44_live.md) says so explicitly rather than returning an empty list.
- **Agent sidecars are never marked.** Only root conversations carry a state; see the algorithm doc for why an agent's mark would be a coincidence of ordering rather than evidence.

With `show_tree::1`, the same rows nest by directory. Shared ancestors become nodes, single-child runs collapse into one segment, and every level is sorted by subtree recency descending:

```
5 projects · 6 conversations · 19 agents · 2 live (1 working, 1 waiting)

  ~/pro/lib
  ├─ assistant_kit                 ○ waiting            1 conv   3 ag  5h ago
▸ │  ├─ claude_storage             ● working            2 conv  12 ag  2h ago
  │  └─ module/claude_storage/docs                      1 conv      ·  1d ago
  ├─ wtools                                             1 conv   4 ag  3d ago
  └─ assistant/kit/-commit                    ⚠ gone    1 conv      ·  6d ago
```

Structural nodes — directories that are ancestors of several projects but hold no sessions of their own — render as a label with empty count columns; `~/pro/lib` above is one. Their `STATUS` cell stays blank too: a structural node owns no session, so it never inherits a child's attachment. Sibling leaves under one node are the common case: a repository entered from several subdirectories becomes several projects, and the tree is what makes that visible. Single-child runs collapse into one segment, so `module/claude_storage/docs` is one node rather than three.

The last row shows what a lossy decode looks like once the directory is gone: `assistant_kit/-commit` was recorded, but with the directory deleted the `_` could no longer be distinguished from a `/`, so it renders as `assistant/kit/-commit` — hence its position as a sibling of `assistant_kit` rather than a child, and hence the marker. Ordering is by recency descending at every level, including among structural nodes (a node sorts by the most recent session anywhere beneath it).

`detail::sessions` uses the full list format. Path header always shown:

```
Found N projects:

~/path/to/project-a: (2 conversations, 12 agents)
  * a1b2c3d4  2h ago  (347 entries)  ● working  [8 agents: 5×Explore, 2×general-purpose, 1×Plan]
  - e5f6a7b8  1d ago  (42 entries)   [4 agents: 3×Explore, 1×general-purpose]

~/path/to/project-b: (1 conversation)
  * c9d0e1f2  3d ago  (2 entries)
```

The `*`/`-` marker keeps its own meaning — most recent in the project — and the liveness tag sits beside it rather than replacing it, because the two are independent: the newest conversation is frequently *not* the attached one, which is exactly the case a recency marker alone reads backwards. The tag precedes `show_topic::`'s free text so a long topic cannot push it off the edge of a terminal.

Family display (`detail::sessions`): agents are grouped by parent session into families. Each root session line shows an inline `[N agents: breakdown]` suffix. Roots with no agents show no bracket suffix. Orphan families (root deleted) use `?` marker. When `agent::` filter is set, family grouping is disabled — flat display.

With `detail::sessions show_tree::1`, agents are tree-indented under their parent:
```
~/path/to/project-a: (2 conversations, 12 agents)
  - a1b2c3d4-e5f6-7890-abcd-ef1234567890  (347 entries)
    ├─ agent-a6061d6e2a0c37a78  Explore  12 entries
    ├─ agent-3f8b2c91ea44d2b10  Explore   8 entries
    └─ agent-7e4a0b23ff129c5a2  general-purpose  42 entries
  - e5f6a7b8-...  (42 entries)
    └─ agent-c1d2e3f4  Explore  15 entries
```

`ids::1` output format (one conversation ID per line, or a bare integer with `count::1`):
```
a1b2c3d4-e5f6-7890-abcd-ef1234567890
e5f6a7b8-1234-5678-90ab-cdef12345678
```

**Display rules:**
- `*` marks the first (most recent) root session; `-` marks the rest
- Short UUID: 36-char UUID IDs are truncated to first 8 chars; non-UUID IDs shown in full
- Zero-byte sessions excluded (startup placeholders)
- Family display: agents grouped by parent; inline `[N agents: N×Type, …]` per root
- Orphan families (no root): `  ? (orphan)  [N agents: breakdown]`
- `limit::N` caps families per project; truncated projects show `... and N more sessions` hint (`detail::sessions` only)
- `show_tree::1` — under `detail::sessions`, agents tree-indented under parent (`├─`/`└─`) with full IDs; under `detail::projects`, projects nested by directory
- `since_days::N` — sessions outside the mtime window dropped before aggregation; a project with no surviving session disappears entirely
- `show_topic::1` — first user message text appended to session lines (newlines flattened, truncated at 90 chars); compact and flat views only, tree view unchanged (`detail::sessions` only)
- `detail::projects` — one row per project under a totals summary line; `limit::` and `show_topic::` do not apply, `show_tree::` does
- `detail::projects` counts — `CONV` counts root conversations, `AGENTS` counts agent sessions across them; zero renders as `·`, and the agents segment of the summary line is omitted entirely at zero
- `detail::projects` ordering — recency descending, both as flat row order and as sibling order at every tree level
- `▸` gutter — marks the project whose path equals the current working directory; absent when cwd matches no listed project
- `⚠ gone` — the decoded path names no existing directory; the path is still shown, but is a reconstruction rather than a verified location
- `detail::projects` paths — always printed in full; rows are never factored against a shared prefix, so each stays usable in `cd`, `grep`, or `project::`. Prefix factoring is what `show_tree::1` provides, via nesting rather than truncation
- `ids::1` — one conversation ID per line, or (with `count::1`) a single bare integer; no path headers

### Algorithms

| File | Relationship |
|------|-------------|
| `../../algorithm/001_agent_session_tracking.md` | Agent session discovery algorithm this command displays |
| `../../algorithm/002_session_liveness.md` | Attached-process inference behind the `STATUS` column, the session tags, and `live::` |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `show_stat::`, `show_tokens::` |
| 2 | [Project Scope](../param_group/02_project_scope.md) | Partial | `path::` (this command's `path::` is the Scope Configuration anchor role, not the Project Scope group's `project::`-pairing role) |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 1 | [`agent::`](../param/01_agent.md) | Boolean | optional |
| 7 | [`min_entries::`](../param/07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | optional |
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 10 | [`project::`](../param/10_project.md) | String | conditional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 13 | [`session::`](../param/13_session.md) | [`SessionFilter`](../type/08_session_filter.md) | optional |
| 18 | [`type::`](../param/18_type.md) | [`ProjectType`](../type/06_project_type.md) | optional |
| 21 | [`count::`](../param/21_count.md) | Boolean | optional |
| 22 | [`limit::`](../param/22_limit.md) | Integer | optional |
| 24 | [`show_tree::`](../param/24_show_tree.md) | Boolean | optional |
| 27 | [`since_days::`](../param/27_since_days.md) | Integer | optional |
| 28 | [`show_topic::`](../param/28_show_topic.md) | Boolean | optional |
| 29 | [`filter::`](../param/29_filter.md) | [`PathSubstring`](../type/04_path_substring.md) | optional |
| 30 | [`detail::`](../param/30_detail.md) | `DetailLevel` | optional |
| 31 | [`ids::`](../param/31_ids.md) | Boolean | optional |
| 44 | [`live::`](../param/44_live.md) | Boolean | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
