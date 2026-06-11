# Parameter :: 26. `desc::`

Controls sort direction for the `.usage` quota table. Each `sort::` strategy has a context-sensitive default; `desc::` overrides it.

- **Default:** context-sensitive (see below)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Override the sort strategy's natural direction.

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

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Sort Control](../param_group/004_sort_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Sort direction override for quota table |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Reverse natural sort order for specific workflows |
