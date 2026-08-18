# Command :: 13. `.usage`

### Scope

- **Purpose**: Specify the `.usage` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.usage`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Implemented in `src/cli/usage.rs`. No other shipped command owns this responsibility — `.projects` (which absorbed `.list`, deprecated) prints session listings only (no metrics), `.count` is contractually a single bare integer (not a table), `.status`'s `show_tokens::1` is one **global** rollup, never per-session.

**Representation Absorption Test** (per [`command_group/readme.md`](../command_group/readme.md), the mandatory gate before adding any new command name): closest candidate is [`.projects`](07_projects.md) — it already implements `scope::`/`path::`/`limit::`, the same discovery machinery `.usage` reuses. Fails both criteria anyway: (1) *identical routine* — `.usage` has its own routine; `projects_routine()`'s output is project-grouped (summary/list/family-tree rendering keyed on project path, with agent-family detection) while `.usage`'s is a flat, session-keyed table with its own numeric/duration formatting (k/M suffixes, s/m/h) that `.projects` has no equivalent of; not reachable by changing `.projects`' parameter defaults. (2) *identical parameter set* — `.usage` registers `depth::`, which `.projects` does not register and has no default-value equivalent for (`.projects`' walk is uncapped). Confirmed as a genuinely new command, not a disguised `.projects` reparameterization.

Print a per-session usage table — turn count, token usage, cache reads, wall-clock duration, the originating command, and the working directory — across one or more sessions in scope. Use this to audit recent activity, compare session cost/length, or find which directory a burst of work happened in.

**Parameters:** `scope::`, `path::`, `depth::`, `limit::`

**Exit:** `0` success (including an empty, zero-row result for non-`local` scopes) | `1` argument error (invalid `scope::`/`depth::`/`limit::` value) | `2` storage error (`scope::local` and cwd has no project; or a storage read error)

**Syntax:**
```bash
claude_storage .usage
claude_storage .usage scope::under path::/home/alice/projects
claude_storage .usage scope::relevant depth::5
claude_storage .usage scope::global limit::20
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Discovery boundary — see per-command semantics below |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Filesystem anchor for scope resolution |
| `depth::` | Integer | optional | `3` | Max path-component distance from `path::` for `under`/`relevant`/`around`; `0` = unbounded; ignored for `local`/`global` |
| `limit::` | Integer | optional | `0` | Max sessions shown, most-recent-first by mtime; `0` = unlimited |

`scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md) — `.usage` is its second real implementer alongside [`.projects`](../command/07_projects.md), with its own default (`local`, not `around`) chosen for cost containment (see Notes). `depth::` is introduced by this command; no other command registers it today. `limit::` is shared with [`.projects`](../command/07_projects.md), extended here from a per-project cap to a flat cap across the whole result set.

**`scope::` semantics for `.usage`** (reuses the canonical [`ScopeValue`](../type/07_scope_value.md) enum and its existing `.projects` meanings unchanged — no redefinition):

| Value | Sessions included | `depth::` applies |
|-------|--------------------|--------------------|
| `local` (default) | Current `path::`-resolved project only | No |
| `relevant` | Projects in the ancestor chain from `path::` up to `/` | Yes |
| `under` | Projects at or under `path::` (subtree) | Yes |
| `around` | Union of `relevant` ∪ `under` (deduplicated) | Yes |
| `global` | All projects in storage; `path::`/`depth::` ignored | No |

**Algorithm (6 steps):**
1. Validate parameters — parse `scope::` (default `local`), `path::` (default cwd), `depth::` (default `3`, non-negative), `limit::` (default `0`, non-negative)
2. Resolve candidate projects — `local`: the single `path::`-resolved project (cwd's project when `path::` omitted), the same resolution `.count`/`.show` already use; `global`: every project from `storage.list_projects()`, no filtering; `under`/`relevant`/`around`: Stage 1 — compare **encoded** project directory names against `encode_path(path::)` (never decoded names, which are lossy — see [`invariant/001_path_encoding.md`](../../invariant/001_path_encoding.md)): `under` keeps names prefixed by the encoded anchor, `relevant` keeps names that are a prefix of the encoded anchor, `around` keeps the union — zero file opens, filesystem listing only
3. Confirm candidates — for `under`/`relevant`/`around` only: open each Stage 1 candidate's session file(s) and read the first line to get the real `Entry.cwd`; confirm the relationship via component-wise path comparison (not string prefix, which mismatches at path-segment boundaries, e.g. `/a/b-c` vs `/a/b`); apply `depth::` by counting path components between `path::` and the confirmed `cwd`, dropping candidates beyond it (`depth::0` = unbounded)
4. Exclude agent sessions — drop sessions with `is_agent_session == true` from every scope, matching the main/agent distinction [`.tail`](../command/12_tail.md) and [`.projects`](../command/07_projects.md) already apply
5. Compute per-session stats — for each remaining session, the same aggregation `Session::stats()` already performs: Turns = `assistant_entries`, In/Out = `total_input_tokens`/`total_output_tokens`, Cache = `total_cache_read_tokens`, Dur = `last_timestamp − first_timestamp`, Dir = `cwd` (new `SessionStats` field — see Notes), Command = the first non-sidechain user entry's text, or `<command-name>` when it was a slash command
6. Sort and render — sort by session mtime descending (most recent first); apply `limit::` cap (`0` = unlimited); render a fixed-width table, `Dir` last and untruncated since it is the one unbounded-width column

**Examples:**
```bash
# Current project's sessions (default scope, cheap)
claude_storage .usage

# All sessions under a directory tree, default depth
claude_storage .usage scope::under path::/data/repos/yrd_review

# Ancestor + current + descendant neighborhood, deeper walk
claude_storage .usage scope::around depth::5

# Whole storage, capped to the 20 most recent sessions
claude_storage .usage scope::global limit::20
```

**Output** (columns: Session, Command, Turns, In, Out, Cache, Dur, Dir):
```
Session   Command                            Turns      In     Out   Cache      Dur  Dir
bf61b676  /role                                 31   44.8k  105.8k   4.8M    5m24s  /data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101
a2201ceb  /role                                 35   55.2k  109.2k   5.0M    4m22s  /data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_144
```
- `Session`: 8-character short form, the same `short_id()` helper [`.projects`](../command/07_projects.md) already uses
- `Command`: truncated to 35 characters with a trailing `…` when longer
- `In`/`Out`/`Cache`: `< 1000` shown as a bare integer; `1000` to `999999` shown as `N.Nk`; `≥ 1000000` shown as `N.NM` (one decimal place) — raw 6-8 digit integers (as [`.status`](../command/01_status.md)'s `show_tokens::1` prints today) would break column alignment across many rows
- `Dur`: `< 60s` → `Ns`; `< 3600s` → `NmNNs`; `≥ 3600s` → `NhNNm`

**Notes:**
- `scope::local` is the default — deliberately, not `around` (the `ScopeValue` type's own default, used by `.projects`). Unlike `.projects`, which only lists sessions (filesystem-cheap), `.usage` opens and parses every candidate session to compute stats — the same unscoped-JSONL-parsing cost that [`.status`](../command/01_status.md)'s own `Fix(issue-015)` already hit ("1903 projects / 2449 sessions / 7 GB" took over two minutes). Broadening beyond `local` is opt-in, exactly like `.status`'s `show_tokens::1`.
- `depth::` exists only for `.usage` — `.projects`'s own `under`/`around`/`relevant` walk is uncapped today. `.usage` needs the cap because its per-candidate cost (open + parse) is materially higher than `.projects`'s filesystem-only listing.
- The `model` column was evaluated and deliberately excluded from this version: `Entry.model` is already parsed per-entry, but `SessionStats` has no aggregate field for it yet. Adding one (first assistant entry's model wins, mirroring the `cwd` idiom below) is a straightforward one-field future addition, not part of this command's initial scope.
- Sessions that were compacted and resumed across multiple calendar days show `Dur` as the full first-to-last wall-clock span, not active runtime — the same caveat applies to any duration derived from `first_timestamp`/`last_timestamp` alone (no separate "active time" field exists in the JSONL).
- Agent/sidechain sessions never appear as their own rows, consistent with [`.tail`](../command/12_tail.md)'s recency fallback and [`.projects`](../command/07_projects.md)'s main/agent distinction.

### Referenced Parameter Groups

| # | Group | Membership | Notes |
|---|-------|------------|-------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | Seventh implementer; own default (`local`) |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 22 | [`limit::`](../param/22_limit.md) | Integer | optional |
| 26 | [`depth::`](../param/26_depth.md) | Integer | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
