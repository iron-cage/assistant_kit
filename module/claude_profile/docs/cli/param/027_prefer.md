# Parameter :: 27. `prefer::`

Selects which weekly quota column is used by sort strategies and recommendation heuristics. Determines whether the overall weekly quota (`7d Left`), the Sonnet-specific weekly quota (`7d(Son)`), or the more constrained of the two is considered.

- **Default:** `any`
- **Constraints:** `any`, `opus`, `sonnet`
- **Purpose:** Tell the sort/recommendation heuristics which model the user intends to run.

**Values:**

| Value | Weekly column used | When to use |
|-------|-------------------|-------------|
| `any` | `min(7d Left, 7d(Son))` | Default — conservative, uses whichever limit is more constrained |
| `opus` | `7d Left` | Running Opus — only overall weekly quota matters |
| `sonnet` | `7d(Son)` | Running Sonnet — Sonnet-specific weekly cap matters |

**Affected heuristics:**
- `sort::renew` secondary key: lowest weekly(prefer) first (ascending)
- `→` recommendation eligibility: `prefer_weekly > 5.0` required

**Examples:**

```text
prefer::any       → min(7d Left, 7d(Son)) — conservative (default)
prefer::opus      → 7d Left — for Opus sessions
prefer::sonnet    → 7d(Son) — for Sonnet sessions

sort::renew prefer::sonnet       → renew tiebreak uses 7d(Son) ascending
sort::renew prefer::opus         → renew tiebreak uses 7d Left ascending
```

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Sort Control](../param_group/004_sort_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Model preference for sort and recommendation heuristics |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Model-aware quota sorting for workflow optimization |
