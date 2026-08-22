# Command :: 4. `.count`

### Scope

- **Purpose**: Specify the `.count` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.count`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Fast counting of projects, sessions, or entries without loading full content. Optimized for performance on large storage (2000+ projects). Use this when you need a number, not a listing.

**Parameters:** `target::`, `project::`, `session::`, `path::`, `scope::`

`scope::` (default `global`) narrows `target::projects` and the `target::sessions`-without-`project::` sum to a discovery boundary around the current directory (see [Scope Configuration](../param_group/05_scope_configuration.md)); `global` reproduces the original unscoped counts exactly, including `target::projects`' fast `count_projects()` path (no per-project load). `scope::` has no effect on the `target::entries`/`target::conversations` targets (both already require an explicit `project::`) or on the context-aware no-argument cwd-shortcut (Algorithm step 1) — neither branch reads it. `path::` remains a real, working parameter here, but overrides the entire storage root (see table below), not a scope anchor.

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .count
claude_storage .count target::sessions project::PROJECT
claude_storage .count target::entries project::PROJECT session::SESSION
claude_storage .count target::conversations project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `target::` | [`TargetType`](../type/11_target_type.md) | optional | `projects` | What to count (`projects`, `sessions`, `entries`, `conversations`) |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | — | Scope to this project |
| `session::` | [`SessionId`](../type/09_session_id.md) | optional | — | Scope to this session |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `global` | Discovery boundary for `target::projects`/`target::sessions`-without-`project::` |
| `path::` | String | optional | `~/.claude/` | Custom storage root override — replaces the entire storage location, not a scope anchor |

`project::` belongs to the [Project Scope group](../param_group/02_project_scope.md). `scope::` belongs to the [Scope Configuration group](../param_group/05_scope_configuration.md) and narrows `target::projects`/`target::sessions`-without-`project::` as described above; `path::` is registered but with storage-root-override semantics, not the group's anchor semantics.

**Algorithm (3 steps):**
1. Context-aware dispatch — no `target::` and no `project::`: count entries in cwd project (matches `.show` default); fall through to global project count if cwd has no project
2. Target-specific counting — `projects`: storage-level count, `scope::`-narrowed when non-`global` (else the fast unscoped path); `sessions`: project-level (requires `project::`, `scope::` ignored) or sums across `scope::`-narrowed projects when `project::` absent; `entries`: session-level (`session::` uses prefix matching for partial UUIDs, Git-style 8-char prefix, consistent with `.show`/`.export`/`.search`) or project-level sum (skips corrupted sessions with warning), `scope::` ignored (already requires `project::`); `conversations`: family grouping count, `scope::` ignored (already requires `project::`)
3. Output bare integer — single number, no formatting, suitable for shell capture (`$(clg .count ...)`)

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

# Count only this directory's own project (scope::local)
claude_storage .count target::projects scope::local
```

**Notes:**
- `target::sessions` requires `project::` to avoid counting all sessions in all projects
- `target::entries` requires both `project::` and `session::`
- `target::conversations` requires `project::` (currently 1:1 with sessions; will differ once chain detection is implemented)
- `session::` matches a leading prefix of the session ID (e.g. the first 8 characters of a UUID), never a substring found elsewhere in the ID (BUG-490)
- `scope::` only affects `target::projects` and `target::sessions`-without-`project::`; `global` (the default) preserves the original unscoped counts and `target::projects`' fast path exactly
- The no-argument cwd-shortcut (Algorithm step 1) never reads `scope::` — even an invalid value like `scope::bogus` has no effect when no `target::`/`project::` is given

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Partial | `agent::`, `min_entries::` |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial (implemented) | `path::` (storage-root override, not an anchor) |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | String | optional |
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | optional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 13 | [`session::`](../param/13_session.md) | [`SessionId`](../type/09_session_id.md) | optional |
| 16 | [`target::`](../param/16_target.md) | [`TargetType`](../type/11_target_type.md) | optional |

### Referenced Command Group

Evaluated against `.show` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify. `count_routine()` (`src/cli/count.rs:24`) has zero cross-calls with `show_routine()` (`src/cli/show.rs:32`). The "matches `.show` default" language above (Algorithm step 1) refers to `count_routine()`'s own doc comment stating its zero-parameter default was deliberately engineered to reproduce `.show`'s CWD-detection default "for UX consistency" (`Fix(issue-003a)`, `src/cli/count.rs:27-34`) — a behavioral-parity design decision independently implemented in each routine (`count_routine()` calls `storage.load_project_for_cwd()` directly at dispatch time; `show_routine()` does the same via its own internal helper), not implementation sharing. Their parameter sets also differ (`target::`/`session::` vs `session_id::`/`show_entries::`/etc.). See [`../command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
