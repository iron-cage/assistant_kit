# Group :: 4. Sort Control

**Parameters:** `sort::`, `desc::`, `prefer::`
**Pattern:** Per-invocation display ordering
**Purpose:** Controls how `.usage` rows are ordered in text output — which heuristic strategy to apply, the sort direction, and which weekly quota column the heuristics reference.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`sort::`](../param/025_sort.md) | `enum` | `name` | Row ordering strategy: `name`, `endurance`, `drain`, `reset` |
| [`desc::`](../param/026_desc.md) | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy |
| [`prefer::`](../param/027_prefer.md) | `enum` | `any` | Weekly quota column for sort heuristics: `any`, `opus`, `sonnet` |

**Used By (1 command):** [`.usage`](../command/006_usage.md#command--9-usage)

**Typical Patterns:**

```bash
# Default: alphabetical — stable for live::1 monitor mode
clp .usage

# Find accounts suitable for a long uninterrupted agent run
clp .usage sort::endurance

# Same, but user knows they're running Sonnet
clp .usage sort::endurance prefer::sonnet

# Drain lowest-quota accounts before their session resets
clp .usage sort::drain

# Use accounts whose quota refills soonest
clp .usage sort::reset

# Reverse the endurance order (worst candidates on top, for inspection)
clp .usage sort::endurance desc::0
```

**Semantic Coherence Test**

> "Does parameter X control **how `.usage` orders rows** (strategy, direction, or column selection for heuristics)?"

All 3 members pass: `sort::` (ordering strategy), `desc::` (sort direction), `prefer::` (which weekly column the sort heuristics reference). `refresh::` fails (fetch retry strategy, not ordering) and is correctly excluded.

**Invariants**

- `desc::` default changes when `sort::` changes — see [../004_parameter_interactions.md#interaction--5](../004_parameter_interactions.md).
- `prefer::` affects `endurance` and `drain` strategies; `reset` does not reference weekly quota directly.
- `sort::` and `desc::` have no effect when `format::json` is specified — JSON array order is always alphabetical.
- Sort order is preserved within each `live::1` refresh cycle; alphabetical `sort::name` is recommended for monitor mode to prevent rows jumping.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — interactions 5, 6, 7 govern sort parameter co-dependencies
- [../../feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) — full strategy algorithm definitions and ACs
