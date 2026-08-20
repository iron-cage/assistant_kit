# Feature: Token-Usage Rollup

### Scope

- **Purpose**: Enable flexible grouped/filtered/sorted/projected token-usage reporting across sessions, projects, models, and days.
- **Responsibility**: Documents the rollup design: grouping dimensions, sort/filter semantics, percent computation, and the core/CLI split.
- **In Scope**: GroupKey/SortKey/SortOrder design, RollupInput/RollupRow/RollupParams shape, `build_rollup()` aggregation algorithm, percent-of-grand-total semantics.
- **Out of Scope**: Column projection and rendering (→ `claude_storage` crate CLI layer), CLI parameter parsing (→ `claude_storage/docs/cli/command/14_rollup.md`).

### Design

`.usage` reports one fixed row shape per session with no grouping, filtering, sorting, or column choice. The rollup engine generalizes that into a flexible reporting primitive: group by session/project/model/day, filter by model substring, sort by any computed column in either direction, and cap the row count — while leaving column *projection* (which fields to print, and in what order) to the CLI layer, exactly as `.usage` already splits `render_row`/`format_tokens` out of its own core computation.

**Pure aggregation, no I/O.** `build_rollup()` takes an already-assembled `&[RollupInput]` slice — one entry per session, each wrapping an already-computed `SessionStats` — and returns `Vec<RollupRow>`. It never touches the filesystem itself; the CLI layer is responsible for walking scope-resolved projects/sessions and assembling `RollupInput` values, mirroring `.usage`'s own `collect_rows` glue. This keeps every grouping/sort/filter path unit-testable with plain synthetic values, with no JSONL fixtures required.

**Grouping dimensions.** `GroupKey::Session` is the finest granularity (closest to `.usage`'s shape, but still sortable/filterable/projectable). `Project` sums every session under a shared `project_label`. `Model` buckets by first-seen model name, falling back to `"unknown"` when absent. `Day` buckets by the `YYYY-MM-DD` prefix of `first_timestamp` via plain string slicing (no date-parsing dependency, consistent with the crate's zero-dependency design — see `001_core_library.md`), also falling back to `"unknown"` when the timestamp is absent.

**Model filter is session-granularity, pre-grouping.** A session whose `stats.model` doesn't match — or is absent while a filter is set — is dropped before it can contribute to any row, regardless of grouping dimension.

**Percent is computed against the full filtered grand total, before `limit` truncates rows.** This keeps "this row is N% of the total" meaningful under a narrow `limit::`: a `limit::5` view still reports each row's true share of everything that survived the model filter, not just of the other 4 rows shown alongside it. The grand-total-is-zero case returns the `0.0` literal directly, never a computed division, so it can never produce `NaN`.

**Depends on `Session::stats()` deduplication.** `RollupRow`'s token sums, `calls`, and `max_context` are only correct because `SessionStats` itself is deduplicated by `message.id` — see the `session_stats_dedup_bug.rs` bug reproducer (issue-038) for the underlying per-API-call-not-per-JSONL-line accounting this engine relies on.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/rollup.rs` | GroupKey, SortKey, SortOrder, RollupInput, RollupRow, RollupParams, build_rollup() |
| source | `../../src/stats.rs` | SessionStats — the deduplicated per-session input this engine aggregates |
| test | `../../tests/rollup_test.rs` | Grouping, model filtering, percent computation, sorting, and `limit` unit tests |
| test | `../../tests/session_stats_dedup_bug.rs` | Bug reproducer (issue-038) for the `message.id` dedup this engine's totals depend on |
| doc | `../api/001_public_api.md` | Public API surface for the rollup engine |
| doc | `claude_storage/docs/cli/command/14_rollup.md` | `.rollup` CLI command contract this engine is built to serve |

### Sources

| File | Notes |
|------|-------|
| `session_stats_dedup_bug.rs` (issue-038) | Bug found while building this feature; motivated `SessionStats::max_context_tokens` and `SessionStats::model` |
