# Parameter Group :: 5. Scope Configuration

### Scope

- **Purpose**: Specify the Scope Configuration parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Scope Configuration.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `scope::`, `path::`

**Pattern:** Discovery scope boundary and anchor

**Purpose:** Together these control the session discovery strategy: `scope::` selects the discovery algorithm and `path::` provides the filesystem anchor for scope resolution.

**Used By:** `.list` (scope:: only — path:: is PathSubstring in this command), `.count`, `.search`, `.show`, `.export`, `.projects` (6 commands total)

**Note on `.list` membership:** `.list` is a partial member — it accepts `scope::` for discovery boundary control, but its `path::` parameter remains a PathSubstring filter (not a StoragePath anchor); cwd is used as the implicit scope anchor in `.list`.

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
| 2 | [`.list`](../command/02_list.md) | Partial | `path::` (used as PathSubstring) | `scope::` only |
| 3 | [`.show`](../command/03_show.md) | Full | — | |
| 4 | [`.count`](../command/04_count.md) | Full | — | |
| 5 | [`.search`](../command/05_search.md) | Full | — | |
| 6 | [`.export`](../command/06_export.md) | Full | — | |
| 7 | [`.projects`](../command/07_projects.md) | Full | — | |
| 8 | [`.project.path`](../command/08_project_path.md) | Partial | `scope::` | `path::` only |
| 9 | [`.project.exists`](../command/09_project_exists.md) | Partial | `scope::` | `path::` only |
| 10 | [`.session.dir`](../command/10_session_dir.md) | Partial | `scope::` | `path::` only |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | Partial | `scope::` | `path::` only |

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
