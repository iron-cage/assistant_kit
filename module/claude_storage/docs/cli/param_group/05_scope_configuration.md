# Parameter Group :: 5. Scope Configuration

### Scope

- **Purpose**: Specify the Scope Configuration parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Scope Configuration.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `scope::`, `path::`

**Pattern:** Discovery scope boundary and anchor

**Purpose:** Together these control the session discovery strategy: `scope::` selects the discovery algorithm and `path::` provides the filesystem anchor for scope resolution.

**Used By:** `.list` (scope:: only — path:: is PathSubstring in this command), `.count`, `.search`, `.show`, `.export`, `.projects`, `.usage` (7 commands total) — all implemented: [`.projects`](../command/07_projects.md), [`.list`](../command/02_list.md), [`.count`](../command/04_count.md), [`.search`](../command/05_search.md), [`.show`](../command/03_show.md), [`.export`](../command/06_export.md), [`.usage`](../command/13_usage.md), each genuinely wired into its routine. See Referenced Commands below for exact per-command status.

**Note on `depth::`:** the discovery model below already anticipates a third parameter (`scope`, `path`, `depth`). [`.usage`](../command/13_usage.md) is the first command to specify it — see [`depth::`](../param/26_depth.md). It is not yet a formal member of this group (only `scope::`/`path::` are); `.projects` has no depth cap today, so `depth::` remains a `.usage`-specific companion until a second implemented consumer justifies formal membership.

**Note on `.list` membership:** `.list` is a partial member by design, not by omission — it accepts `scope::` for discovery boundary control (implemented) while keeping its pre-existing `path::` as a PathSubstring filter, not a StoragePath anchor. `.list` never gains a second, anchor-role `path::`; the two parameters compose (`scope::` narrows discovery, `path::` substring-filters the result).

**Semantic Coherence Test:**
- "Does `scope::` control how session discovery is bounded?" → YES
- "Does `path::` control where session discovery is anchored?" → YES

**Why NOT `session::`, `agent::`, `min_entries::`:**
- Those parameters filter *which sessions* appear after discovery
- These parameters control *what gets discovered* (where and how)
- Different semantic layer: discovery configuration vs result filtering

**Scope × Path interaction:**

| Scope | Path semantics | Direction |
|-------|----------------|-----------|
| `local` | Starting directory to look up (default: cwd) | ↑ |
| `relevant` | Starting point for ancestor walk (default: cwd) | ↑ |
| `under` | Root of subtree to descend (required when non-cwd) | ↓ |
| `global` | Ignored (all projects regardless of path) | all |
| `around` | Bidirectional anchor: ancestor walk + subtree (default: cwd) | ↑↓ |

The `scope` + `path` pair uses a consistent discovery model across tools (scope, path, depth).

**Parameter Details:**

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | Discovery strategy: `local`\|`relevant`\|`under`\|`global`\|`around` | `around` |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | Filesystem anchor for scope resolution | cwd |

**Examples:**
```bash
.projects scope::around
.projects scope::local
.projects scope::relevant
.projects scope::under path::/home/alice/projects
.projects scope::global
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 2 | [`.list`](../command/02_list.md) | Partial | `path::` (used as PathSubstring) | `scope::` implemented — narrows `type::all` project discovery |
| 3 | [`.show`](../command/03_show.md) | Full | — | Implemented — `scope::`/`path::` narrow the session lookup when `session_id::` given without `project::` |
| 4 | [`.count`](../command/04_count.md) | Partial | `path::` (registered, but as a storage-root override — not a scope anchor) | `scope::` implemented — narrows `target::projects`/`target::sessions`-without-`project::` |
| 5 | [`.search`](../command/05_search.md) | Full | — | Implemented — `scope::`/`path::` narrow project discovery when `project::` is absent |
| 6 | [`.export`](../command/06_export.md) | Full | — | Implemented — `scope::`/`path::` narrow the source-session lookup when `project::` is absent |
| 7 | [`.projects`](../command/07_projects.md) | Full | — | Implemented |
| 13 | [`.usage`](../command/13_usage.md) | Full | — | Implemented — `scope::`/`path::` bound the usage-table aggregation; own `depth::` companion |
| 8 | [`.project.path`](../command/08_project_path.md) | Partial | `scope::` | `path::` only |
| 9 | [`.project.exists`](../command/09_project_exists.md) | Partial | `scope::` | `path::` only |
| 10 | [`.session.dir`](../command/10_session_dir.md) | Partial | `scope::` | `path::` only |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | Partial | `scope::` | `path::` only |
| 12 | [`.tail`](../command/12_tail.md) | Partial | `scope::` | `path::` only |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | cwd | Filesystem anchor for scope resolution |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | `around` | Discovery strategy selector |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
