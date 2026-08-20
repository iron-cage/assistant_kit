# Parameter Group :: 5. Scope Configuration

### Scope

- **Purpose**: Specify the Scope Configuration parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Scope Configuration.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `scope::`, `path::`

**Pattern:** Discovery scope boundary and anchor

**Purpose:** Together these control the session discovery strategy: `scope::` selects the discovery algorithm and `path::` provides the filesystem anchor for scope resolution.

**Used By:** `.list` (deprecated; scope:: only — path:: was PathSubstring in this command), `.count`, `.search`, `.show`, `.export`, `.projects`, `.usage`, `.rollup` (8 commands total, 1 deprecated) — all implemented: [`.projects`](../command/07_projects.md), [`.list`](../command/02_list.md) (deprecated), [`.count`](../command/04_count.md), [`.search`](../command/05_search.md), [`.show`](../command/03_show.md), [`.export`](../command/06_export.md), [`.usage`](../command/13_usage.md), [`.rollup`](../command/14_rollup.md), each genuinely wired into its routine. See Referenced Commands below for exact per-command status.

**Note on `depth::`:** the discovery model below already anticipates a third parameter (`scope`, `path`, `depth`). [`.usage`](../command/13_usage.md) was the first command to specify it (see [`depth::`](../param/26_depth.md)); [`.rollup`](../command/14_rollup.md) is now a second, reusing the identical parsing/validation/depth-walk code unchanged. This is precisely the "second implemented consumer" condition this note originally deferred formal membership on — promoting `depth::` to a formal third group member is now a live candidate, but doing so would reclassify every other Full member below that doesn't implement it (`.show`, `.search`, `.export`, `.projects`) as Partial instead, a cascading change to their own command docs that is out of scope for `.rollup`'s own addition. `depth::` remains a `.usage`/`.rollup`-specific companion for now; formal promotion is a separate decision for whoever next revisits this group's membership model.

**Note on `.list` membership (historical, `.list` deprecated):** `.list` was a partial member by design, not by omission — it accepted `scope::` for discovery boundary control while keeping its own `path::` as a PathSubstring filter, not a StoragePath anchor, so the two composed (`scope::` narrowed discovery, `path::` substring-filtered the result) despite `path::` meaning something different than on every other member of this group. `.projects` resolves that naming collision cleanly: `scope::`/`path::` keep their group-standard anchor meaning, and the substring-filter role moved to a distinctly-named parameter, [`filter::`](../param/29_filter.md) — no more overloaded `path::`.

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
| 2 | [`.list`](../command/02_list.md) (deprecated) | Partial | `path::` (used as PathSubstring) | Historical; superseded by `.projects`' `scope::` + [`filter::`](../param/29_filter.md) |
| 3 | [`.show`](../command/03_show.md) | Full | — | Implemented — `scope::`/`path::` narrow the session lookup when `session_id::` given without `project::` |
| 4 | [`.count`](../command/04_count.md) | Partial | `path::` (registered, but as a storage-root override — not a scope anchor) | `scope::` implemented — narrows `target::projects`/`target::sessions`-without-`project::` |
| 5 | [`.search`](../command/05_search.md) | Full | — | Implemented — `scope::`/`path::` narrow project discovery when `project::` is absent |
| 6 | [`.export`](../command/06_export.md) | Full | — | Implemented — `scope::`/`path::` narrow the source-session lookup when `project::` is absent |
| 7 | [`.projects`](../command/07_projects.md) | Full | — | Implemented |
| 13 | [`.usage`](../command/13_usage.md) | Full | — | Implemented — `scope::`/`path::` bound the usage-table aggregation; own `depth::` companion |
| 14 | [`.rollup`](../command/14_rollup.md) | Full | — | Implemented — `scope::`/`path::` bound the grouped-table aggregation; same `depth::` companion as `.usage` |
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
