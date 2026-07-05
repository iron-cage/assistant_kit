# Group: 4. Sort Control

**Parameters:** `sort::`, `desc::`, `prefer::`
**Pattern:** Per-invocation display ordering and recommendation control
**Purpose:** Controls how `.usage` and `.accounts` rows are ordered and which account appears in the footer recommendation. `sort::` drives both row ordering and the footer recommendation — single parameter, no separate `next::`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`sort::`](../param/025_sort.md) | `enum` | `renew` | Row ordering strategy AND footer recommendation: `name`, `renew`, `renews` |
| [`desc::`](../param/026_desc.md) | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy |
| [`prefer::`](../param/027_prefer.md) | `enum` | `any` | Weekly quota column for sort heuristics: `any`, `opus`, `sonnet` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | All 3 sort control params |
| 2 | [`.accounts`](../command/001_account.md#command-3-accounts) | All 3 sort control params |

**Typical Patterns:**

```bash
# Default: sort::renew — soonest quota refill; recommends soonest-refill account in footer
clp .usage

# Alphabetical for live monitor stability
clp .usage sort::name

# Soonest billing renewal first
clp .usage sort::renews

# User knows they're running Sonnet — select weekly column accordingly
clp .usage sort::renew prefer::sonnet

# Reverse the sort order (worst candidates on top, for inspection)
clp .usage sort::renew desc::1
```

**Semantic Coherence Test**

> "Does parameter X control **how `.usage` or `.accounts` orders rows** (strategy, direction, or column selection for heuristics)?"

All 3 members pass: `sort::` (ordering strategy + footer recommendation), `desc::` (sort direction), `prefer::` (which weekly column the sort heuristics reference). `refresh::` fails (fetch retry strategy, not ordering) and is correctly excluded.

**Invariants**

- `desc::` default changes when `sort::` changes — see [../004_parameter_interactions.md#interaction--5](../004_parameter_interactions.md).
- `prefer::` affects `renew` (secondary key) strategy.
- `sort::` and `desc::` have no effect when `format::json` is specified — JSON array order is always alphabetical.
- Sort order is preserved within each `live::1` refresh cycle; alphabetical `sort::name` is recommended for monitor mode to prevent rows jumping.
- `sort::` drives both row ordering and the footer recommendation — no separate `next::` parameter.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — interactions 5, 6, 7 govern sort parameter co-dependencies
- [../../feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) — full strategy algorithm definitions, footer recommendation, and ACs

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | `sort::`, `desc::`, `prefer::` for quota ordering and recommendation |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | `sort::` for consistent next-account extraction |
