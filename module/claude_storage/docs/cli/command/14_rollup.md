# Command :: 14. `.rollup`

### Scope

- **Purpose**: Specify the `.rollup` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.rollup`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`), aggregation algorithm internals (→ `claude_storage_core/src/rollup.rs`).

Implemented in `src/cli/rollup.rs`; all grouping, filtering, sorting, and percent computation is delegated to `claude_storage_core::rollup::build_rollup()` (`claude_storage_core/src/rollup.rs`) — a pure function with no filesystem or CLI dependency, unit-tested independently of this command (see `claude_storage_core/tests/`). `src/cli/rollup.rs` itself only walks scope-resolved sessions into `RollupInput`s, parses the 5 new parameters below, and renders the chosen column projection — it duplicates none of the core engine's grouping/sort/filter logic. This split places the aggregation logic at the leaf of the dependency tree (`claude_storage_core`, which nothing in this crate depends on) and keeps the CLI-facing file a thin routine, the same core/CLI division [`.usage`](13_usage.md) already establishes (`Session::stats()` there, `build_rollup()` here).

**Representation Absorption Test** (per [`command_group/readme.md`](../command_group/readme.md), the mandatory gate before adding any new command name): closest candidate is [`.usage`](13_usage.md) — it already implements `scope::`/`path::`/`depth::`/`limit::`, the same discovery machinery `.rollup` reuses verbatim. Fails both criteria: (1) *identical routine* — `rollup_routine()` delegates every grouping/sort/filter/percent step to `claude_storage_core::rollup::build_rollup()`, which `usage_routine()` never calls; `.usage`'s own rendering is a fixed, ungroupable, unsortable, unprojectable per-session table with `Command`/`Dir` columns `.rollup` has no equivalent of, while `.rollup`'s output is grouped/sorted/column-projected with a `Percent` column `.usage` has no equivalent of. Not reachable by changing `.usage`'s parameter defaults — `.usage` has no aggregation code path at all to redirect. (2) *identical parameter set* — `.rollup` registers `group::`/`sort::`/`order::`/`model::`/`columns::`, none of which `.usage` registers, and has no equivalent of `.usage`'s fixed `Command`/`Dur` columns. Confirmed as a genuinely new command, not a disguised `.usage` reparameterization.

Print a flexible, aggregated token-usage table — grouped by session, project, model, or calendar day; filtered by model substring; sorted by any computed column in either direction; and projected to only the columns you want. Use this to compare cost across projects or models, find which day burned the most tokens, or audit context-window usage (`MaxCtx`) across a fleet of sessions — the cross-sectional counterpart to [`.usage`](13_usage.md)'s fixed per-session detail view.

**Parameters:** `group::`, `sort::`, `order::`, `model::`, `columns::`, `scope::`, `path::`, `depth::`, `limit::`

**Exit:** `0` success (including an empty, zero-row result for non-`local` scopes, or when `model::` filters out every candidate session — the header still prints, and the zero-total `percent` branch renders `0.0`, never `NaN`) | `1` argument error (invalid `group::`/`sort::`/`order::`/`columns::`/`scope::`/`depth::`/`limit::` value) | `2` storage error (`scope::local` and cwd has no project; or a storage read error)

**Syntax:**
```bash
claude_storage .rollup
claude_storage .rollup group::project sort::calls order::asc
claude_storage .rollup model::opus columns::group,total,percent
claude_storage .rollup scope::global group::day limit::10
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `group::` | String enum | optional | `session` | Aggregation dimension — see grouping semantics below |
| `sort::` | String enum | optional | `total` | Column the grouped rows are sorted by |
| `order::` | String enum | optional | `desc` | Sort direction |
| `model::` | String | optional | none | Case-insensitive substring filter against each session's recorded model, applied before grouping |
| `columns::` | String (comma list) | optional | see [Column Projection](#column-projection) below | Which columns to print, and in what order |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Discovery boundary — reused unchanged from [`.usage`](13_usage.md) |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Filesystem anchor for scope resolution — reused unchanged from [`.usage`](13_usage.md) |
| `depth::` | Integer | optional | `3` | Max path-component distance from `path::` for `under`/`relevant`/`around`; `0` = unbounded; ignored for `local`/`global` — reused unchanged from [`.usage`](13_usage.md) |
| `limit::` | Integer | optional | `0` | Max **grouped rows** shown after sorting; `0` = unlimited — see Notes for why this differs from [`.usage`](13_usage.md)'s flat per-session cap |

`scope::`/`path::`/`depth::`/`limit::` are byte-for-byte the same parsing, validation, and (for `scope::`/`path::`/`depth::`) resolution code [`.usage`](13_usage.md) uses — see that command's own doc for the full `scope::` semantics table and depth-walk algorithm, not repeated here. `group::`/`sort::`/`order::`/`model::`/`columns::` are introduced by this command; no other command registers them today.

**`group::` semantics** (aggregation dimension — reuses `claude_storage_core::GroupKey`):

| Value | Row shown per... |
|-------|-------------------|
| `session` (default) | Each session individually — finest granularity, closest to [`.usage`](13_usage.md)'s own shape, but still sortable/projectable/filterable unlike that fixed command |
| `project` | Each project (session's recorded `cwd`), summing every session under it |
| `model` | Each distinct model name; sessions with no recorded model group under `unknown` |
| `day` | Each calendar day (`first_timestamp`'s `YYYY-MM-DD`, UTC as recorded); sessions with no timestamp group under `unknown` |

**`sort::` semantics** (reuses `claude_storage_core::SortKey` — always operates on already-aggregated row totals, sorting happens after grouping, never before):

| Value | Sorts rows by |
|-------|----------------|
| `total` (default) | `input + output + cache` |
| `input` | Fresh (non-cached) input tokens |
| `output` | Generated output tokens |
| `cache` | `cache_read + cache_creation` combined |
| `max_context` | Largest single call's context size across the row's contributing sessions |
| `calls` | Number of deduplicated assistant turns |
| `sessions` | Number of distinct contributing sessions |
| `group` | Lexicographic by the row's group label |

Rows exactly tied on `sort::`'s chosen metric break the tie deterministically by ascending group
label, regardless of `order::` — this guarantees byte-identical output across repeated invocations
against unchanged data (`Fix(BUG-529)`; previously ties fell back to `HashMap` iteration order,
which is process-randomized and produced a different row order on every run).

**`order::` semantics** (reuses `claude_storage_core::SortOrder`):

| Value | Direction |
|-------|-----------|
| `desc` (default) | Largest/last first |
| `asc` | Smallest/first first |

**`model::` semantics:** Case-insensitive substring match against each session's recorded `stats.model` (the same `StringMatcher` mechanism [`.projects`](07_projects.md)'s `filter::` uses against paths — see `claude_storage_core/src/filter.rs` — but applied to a different field; no dedicated type doc, matching the [`.usage`](13_usage.md)-precedent of not creating a type doc for a single-command constrained value; see [`37_model.md`](../param/37_model.md)). Applied at session granularity **before** grouping — a non-matching session is dropped entirely, including from the `percent` denominator (see Notes).

**Column Projection (`columns::`):** A comma-separated, order-preserving list, e.g. `columns::group,total,calls`. Every column but `rank` is always computed internally regardless of projection — `columns::` is a pure display concern, matching [`.usage`](13_usage.md)'s own core/CLI split (the core engine always populates every `RollupRow` field; only the CLI layer decides which to print). `rank` is the one exception: a display-only position the CLI synthesizes from each row's place in the final output, not a `RollupRow` field (see the table's note below).

| Key | Header | In default set? |
|-----|--------|:---:|
| `rank` | `Rank` | — |
| `group` | `Session` / `Project` / `Model` / `Day` — tracks `group::` | ✓ |
| `project` | `Project` | ✓ under `group::session` only |
| `sessions` | `Sessions` | ✓ |
| `calls` | `Calls` | ✓ |
| `input` | `Input` | ✓ |
| `output` | `Output` | ✓ |
| `cache` | `Cache` | ✓ |
| `cache_write` | `CacheW` | — |
| `cache_read` | `CacheR` | — |
| `max_context` | `MaxCtx` | ✓ |
| `total` | `Total` | ✓ |
| `percent` | `Pct` | ✓ |
| `first` | `First` | — |
| `last` | `Last` | — |

Default (`columns::` omitted) depends on `group::` (`Fix(BUG-544)`):

| `group::` | Default column list |
|-----------|---------------------|
| `session` | `group,project,sessions,calls,input,output,cache,max_context,total,percent` |
| `project` / `model` / `day` | `group,sessions,calls,input,output,cache,max_context,total,percent` |

Either way: every count/token metric, including `max_context` (the "window size" metric this command exists partly to surface), but not the verbose `first`/`last` timestamps, nor the opt-in `rank`/`cache_write`/`cache_read` columns (see Notes). `project` joins the default set under `group::session` alone, because there the group label is a bare 8-character session id that names no directory — the other three groupings already carry their dimension in the group label itself.

`rank` and the `cache_write`/`cache_read` split are opt-in only (`Fix(BUG-530)`) — request them explicitly, e.g. `columns::rank,group,total,cache_write,cache_read`. `rank` is a display-only position, synthesized by the CLI after `sort::`/`order::`/`limit::` have all already applied — it has no `RollupRow` field and cannot be used as a `sort::` value (same precedent as `first`/`last`, which are likewise column-only). `cache_write`/`cache_read` are simply the two components `cache` already sums (`RollupRow.cache_creation`/`RollupRow.cache_read`, both computed by the core engine regardless of projection) exposed as separate columns — `cache_write + cache_read` always equals `cache`.

`project` (`Fix(BUG-544)`) is the second CLI-synthesized column after `rank`, and for the same structural reason: `RollupRow` carries only the group label, so a row's owning project is resolved from the pre-aggregation `RollupInput`s (`session_id → project_label`) rather than read off the row. It renders the session's recorded `cwd` under `group::session`, the group label itself under `group::project`, and `-` under `group::model`/`group::day`, where one row can span many projects and no single label would be truthful. Like `rank`, it cannot be used as a `sort::` value. An explicit `columns::` list is honoured verbatim under every grouping, so `columns::project` is always available as an opt-in even where it is not a default.

**Algorithm (9 steps):**
1. Validate parameters, in order — `depth::` (default `3`, non-negative), `limit::` (default `0`, non-negative), `group::` (default `session`), `sort::` (default `total`), `order::` (default `desc`), `columns::` (default: the `group::`-dependent set above — 10 columns under `group::session`, 9 otherwise, out of 15 valid keys total), `model::` (no validation — any string), `scope::` (default `local`), `path::` (default cwd) — identical parsing/error-message contract to [`.usage`](13_usage.md) for the four shared parameters
2. Resolve candidate projects — reuses `resolve_scoped_projects()` unchanged from [`.usage`](13_usage.md); `scope::local` with zero resolved projects exits 2 immediately (before any further work), matching [`.usage`](13_usage.md)'s own bypass
3. Apply the depth filter (`under`/`relevant`/`around` only) — reuses `beyond_depth()`/`component_distance()` unchanged from [`.usage`](13_usage.md); ignored for `local`/`global`
4. Walk every candidate project's non-agent sessions into `RollupInput`s — one per session, carrying `session_id`, `project_label` (the session's own recorded `cwd`, falling back to `"unknown"` — never the lossy-encoded storage directory name), and its already-deduplicated `SessionStats`
5. Filter by `model::` (if set) — a session whose `stats.model` doesn't match (or is absent while a filter is set) is dropped before it can contribute to any row
6. Group the surviving sessions by `group::`'s dimension, summing `sessions`/`calls`/`input`/`output`/`cache_read`/`cache_creation` counts, tracking the running max of `max_context`, and widening each row's `first`/`last` timestamp span
7. Compute `percent` per row against the grand total of the **entire filtered** result set (every group that survives `model::`, before `limit::` truncates rows) — see Notes
8. Sort by `sort::`/`order::`, then apply `limit::` as a flat cap on the **grouped row count** (not the raw session count — see Notes)
9. Render the header — labelling the group column with the active `group::` dimension — and one line per surviving row, using only the columns named in `columns::`; a projected `project` column resolves through the `session_id → project_label` map captured at step 4, before step 6 aggregated the inputs away

**Examples:**
```bash
# Default: one row per session, sorted by total tokens descending
claude_storage .rollup

# Cost per project, most expensive first
claude_storage .rollup group::project

# Which model burned the most calls?
claude_storage .rollup group::model sort::calls

# Busiest days by session count, oldest activity first
claude_storage .rollup group::day sort::sessions order::asc

# Compact cost-only view for a specific model
claude_storage .rollup model::opus columns::group,total,percent

# Top 10 rows, whole storage
claude_storage .rollup scope::global group::project limit::10

# Leaderboard: rank, split cache read/write, project — top 5 by total tokens
claude_storage .rollup group::project limit::5 columns::rank,total,percent,input,output,cache_write,cache_read,calls,group
```

**Output** (default columns, `group::session`):
```
Session                   Project                   Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct
aaaaaaaa                  …/pro/lib/yrd_core/api           1       4       500       300       200       700      1.0k   83.3%
bbbbbbbb                  …/pro/lib/yrd_core/cli           1       2       100        50        50       150       200   16.7%
```
- Group column header: the active `group::` dimension — `Session`, `Project`, `Model`, or `Day` (`Fix(BUG-544)`), never a constant `Group`
- Group column value: for `group::session`, the 8-character short form (same `short_id()` helper [`.usage`](13_usage.md)/[`.projects`](07_projects.md) already use); for other `group::` values, the raw label (project cwd / model name / `YYYY-MM-DD`), truncated to the column's fixed width with a trailing `…` when longer
- `Project`: the session's recorded `cwd`, truncated from the **left** with a leading `…` when longer than the column — the mirror image of every other column's truncation, because sibling project directories share long absolute prefixes and it is the path's tail that distinguishes them
- Numeric columns (`Sessions`/`Calls`/`Input`/`Output`/`Cache`/`MaxCtx`/`Total`): right-aligned, fixed width regardless of which `columns::` are chosen
- `Input`/`Output`/`Cache`/`MaxCtx`/`Total`: `< 1000` shown as a bare integer; `1000` to `999999` shown as `N.Nk`; `≥ 1000000` shown as `N.NM` (one decimal place) — identical formatting rule to [`.usage`](13_usage.md)'s own `In`/`Out`/`Cache` columns
- `Pct`: one decimal place, e.g. `83.3%`
- `First`/`Last` (when projected via `columns::`): raw ISO-8601 timestamp, or `-` when the row has none
- `Rank` (when projected via `columns::`): the row's 1-indexed position among the rows actually printed — always `1, 2, 3, …` top to bottom regardless of which `sort::`/`order::` produced that order, and reflecting `limit::`'s truncation (a `limit::5` view's ranks run `1`–`5`, never referencing rows cut by the limit)
- `CacheW`/`CacheR` (when projected via `columns::`): `cache_write`/`cache_read`, formatted identically to `Cache`; always sum to `Cache`'s value for the same row

**Notes:**
- **`limit::`'s semantics differ from [`.usage`](13_usage.md)'s.** There, the cap is flat across raw sessions (no grouping exists to cap within). Here, `limit::` caps the **grouped, sorted row count** — a third distinct semantic alongside [`.projects`](07_projects.md)'s per-project cap and `.usage`'s flat-per-session cap (see [`22_limit.md`](../param/22_limit.md)). `group::project limit::10` shows the 10 highest-total projects, not the 10 most recent sessions.
- **`percent` is computed against the full filtered set, not the visible one.** A `limit::5` view still reports each row's true share of everything `model::` let through — not just of the other 4 rows shown alongside it. This means the visible rows' `Pct` values do not necessarily sum to `100.0%` when `limit::` is active.
- **`model::` filters before grouping, not after.** A session that doesn't match is invisible to every stage downstream, including the `percent` denominator — filtering out a heavy non-matching session raises the surviving rows' percentages, it does not just hide a row.
- `max_context` (`MaxCtx` column) has no [`.usage`](13_usage.md) equivalent — that command's own Notes section explicitly deferred a `model` aggregate field as a "straightforward one-field future addition, not part of this command's initial scope"; `.rollup` is that later addition, applied to both `model` (as a `group::` dimension) and context-window size (as a sortable/projectable column), motivated by `Session::stats()` already carrying `max_context_tokens`/`model` fields from `Fix(issue-038)`.
- Agent/sidechain sessions never contribute a row, consistent with [`.usage`](13_usage.md)'s and [`.projects`](07_projects.md)'s own main/agent distinction.
- Group-column truncation for non-session groupings can shorten a long project path with a trailing `…`, and the `Project` column truncates from the left — when the exact untruncated path matters, cross-reference with [`.usage`](13_usage.md)'s own `Dir` column, which is never truncated.

### Referenced Parameter Groups

| # | Group | Membership | Notes |
|---|-------|------------|-------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | Eighth implementer; same default (`local`) as [`.usage`](13_usage.md) |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 22 | [`limit::`](../param/22_limit.md) | Integer | optional |
| 26 | [`depth::`](../param/26_depth.md) | Integer | optional |
| 34 | [`group::`](../param/34_group.md) | String enum | optional |
| 35 | [`sort::`](../param/35_sort.md) | String enum | optional |
| 36 | [`order::`](../param/36_order.md) | String enum | optional |
| 37 | [`model::`](../param/37_model.md) | String | optional |
| 38 | [`columns::`](../param/38_columns.md) | String (comma list) | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
