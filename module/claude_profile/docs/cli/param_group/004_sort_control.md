# Group :: 4. Sort Control

**Parameters:** `sort::`, `desc::`, `prefer::`, `next::`
**Pattern:** Per-invocation display ordering and recommendation control
**Purpose:** Controls how `.usage` rows are ordered in text output and which recommendation strategy selects the `→` Next account in the footer.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`sort::`](../param/025_sort.md) | `enum` | `reset` | Row ordering strategy: `reset`, `name`, `endurance`, `drain` |
| [`desc::`](../param/026_desc.md) | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy |
| [`prefer::`](../param/027_prefer.md) | `enum` | `any` | Weekly quota column for sort heuristics: `any`, `opus`, `sonnet` |
| [`next::`](../param/032_next.md) | `enum` | `all` | Recommendation strategy: `all` (multi-strategy footer), `session`, `endurance`, `drain`, `reset` |

**Used By (1 command):** [`.usage`](../command/006_usage.md#command--9-usage)

**Typical Patterns:**

```bash
# Default: sort::reset — soonest quota refill on top
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

All 4 members pass: `sort::` (ordering strategy), `desc::` (sort direction), `prefer::` (which weekly column the sort heuristics reference), `next::` (which recommendation strategy populates the `→` marker and footer). `refresh::` fails (fetch retry strategy, not ordering) and is correctly excluded.

**Invariants**

- `desc::` default changes when `sort::` changes — see [../004_parameter_interactions.md#interaction--5](../004_parameter_interactions.md).
- `prefer::` affects `endurance` and `drain` strategies; `reset` does not reference weekly quota directly.
- `sort::` and `desc::` have no effect when `format::json` is specified — JSON array order is always alphabetical.
- Sort order is preserved within each `live::1` refresh cycle; alphabetical `sort::name` is recommended for monitor mode to prevent rows jumping.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — interactions 5, 6, 7 govern sort parameter co-dependencies
- [../../feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) — full strategy algorithm definitions and ACs
- [../../feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) — `next::` recommendation algorithm definitions and ACs
