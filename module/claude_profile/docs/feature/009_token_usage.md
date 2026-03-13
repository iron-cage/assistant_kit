# Feature: Token Usage Reporting

### Scope

- **Purpose**: Surface locally-cached token consumption data from `stats-cache.json` for the 7-day window.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command (FR-14).
- **In Scope**: 7-day aggregation from stats-cache.json, per-model sort, v::0/1/2/json output.
- **Out of Scope**: Live API-based utilization percentages or reset times (server-side only, require network — see FR-18 for future work); rate-limit tier percentages (not in stats-cache.json).

### Design

`claude_profile` CLI must provide a `.usage` command that reads `~/.claude/stats-cache.json` and reports per-model token totals for the 7-day window ending at `lastComputedDate`.

**Algorithm:**
1. Read `stats-cache.json` from `ClaudePaths::stats_file()`.
2. Parse `lastComputedDate` (error if missing or not a string).
3. Compute `period_start` = `lastComputedDate` − 6 days.
4. Aggregate `dailyModelTokens[]` entries where `date ∈ [period_start, lastComputedDate]`.
5. Sort models descending by total tokens.
6. Report via verbosity level.

**Verbosity levels:**
- `v::0`: total token count only (bare number)
- `v::1` (default): table with model name, total tokens, percentage of total
- `v::2`: v::1 table + daily breakdown per model
- `format::json`: structured JSON with period metadata and per-model array

**Error handling:**
- `HOME` unset → `InternalError`
- `stats-cache.json` missing → `InternalError` with actionable path
- Malformed JSON → `InternalError`
- `lastComputedDate` absent → `InternalError`
- Individual entries with missing `date` or `tokensByModel` → skipped silently (no error)

**Data note:** `stats-cache.json` reports historical token counts only. Live 5-hour and 7-day utilization percentages are server-side data available only via `anthropic-ratelimit-unified-*` response headers at runtime — not in this file.

### Acceptance Criteria

- **AC-01**: `.usage` reads entries within the 7-day window ending at `lastComputedDate`.
- **AC-02**: Models are sorted descending by total token count.
- **AC-03**: Entries with missing `date` or `tokensByModel` are silently skipped.
- **AC-04**: `format::json` returns valid JSON with period and model data.
- **AC-05**: Missing or malformed `stats-cache.json` exits 2 with an actionable error naming the file path.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | Token usage parsing, aggregation, and formatting |
| source | `src/commands.rs` | `usage_routine()` — CLI handler |
| test | `tests/usage_test.rs` | 7-day window, model sort, JSON output tests |
| doc | [013_account_limits.md](013_account_limits.md) | FR-18 future command for live rate-limit data |
| doc | [cli/commands.md](../cli/commands.md#command--10-usage) | CLI command specification |
