# Parameter :: 26. `desc::`

Controls sort direction for the `.usage` quota table. Each `sort::` strategy has a context-sensitive default; `desc::` overrides it.

- **Type:** `bool`
- **Default:** context-sensitive (see below)
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Override the sort strategy's natural direction.
- **Group:** Sort Control

**Context-sensitive defaults:**

| `sort::` value | `desc::` default | Meaning |
|----------------|-----------------|---------|
| `name` | `0` | Ascending A→Z |
| `endurance` | `1` | Best-qualified on top |
| `drain` | `0` | Drain targets on top |
| `reset` | `0` | Soonest reset on top |

**Examples:**

```text
desc::0   → ascending (or strategy's natural ascending direction)
desc::1   → descending (or strategy's natural descending direction)

sort::name desc::1       → Z→A
sort::drain desc::1      → freshest accounts on top (reversed)
sort::endurance desc::0  → worst candidates on top (reversed)
```
