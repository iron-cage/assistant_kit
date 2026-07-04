# Parameter: 26. `desc::`

Controls sort direction for the `.usage` quota table. Each `sort::` strategy has a context-sensitive default; `desc::` overrides it.

- **Default:** context-sensitive (see below)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Override the sort strategy's natural direction.

**Context-sensitive defaults:**

| `sort::` value | `desc::` default | Meaning |
|----------------|-----------------|---------|
| `name` | `0` | Ascending A→Z |
| `renew` | `0` | Soonest quota event on top |
| `renews` | `0` | Soonest billing renewal on top |

**Examples:**

```text
desc::0   → ascending (or strategy's natural ascending direction)
desc::1   → descending (or strategy's natural descending direction)

sort::name desc::1       → Z→A
sort::renew desc::1      → latest quota event on top (reversed)
sort::renews desc::1     → latest billing renewal on top (reversed)
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
