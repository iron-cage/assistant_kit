# Verb: inspect

Performs a unified live account diagnostic — identity, subscription, org, and quota utilization for a named account. Fetches data from three API endpoints (`GET /api/oauth/account`, `GET /api/oauth/claude_cli/roles`, `GET /api/oauth/usage`), consolidates the results, and reports identity fields (tagged_id, uuid, email, name), subscription tier, billing type, membership list, capabilities, rate-limit tier, and 5h/7d/Sonnet quota utilization with reset countdowns. Does not modify local state unless `refresh::1` triggers a token refresh subprocess.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.inspect` | Yes | No |

### Behavioral Contract

**Pre-conditions:**
- Named account (or active account if `name::` omitted) credentials accessible in credential store
- Network reachable; all three API endpoints accessible
- `$HOME` environment variable set

**Post-conditions:**
- Identity, subscription, billing, membership, capability, and quota utilization data reported
- No local files written or modified (unless `refresh::1` and token expired)

**Side effects:**
- Makes HTTP requests to endpoints 002, 005, and 001
- If `refresh::1` and token is expired, isolated subprocess attempts token refresh before fetching
- No persistent state changes from the diagnostic reads themselves

### Idempotency

**Yes.** Pure diagnostic read across three live API endpoints (002, 005, 001). Repeated calls return current API state; no side effects accumulate.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to inspect; defaults to active account | No |
| `refresh::` | Attempt token refresh if expired before fetching | No |
| `format::` | Output format (`text` or `json`) | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Reads state.** Fetches from three live API endpoints; no local writes. Account lifecycle state unchanged.

```
[saved/active] --account.inspect--> [saved/active]  (state unchanged; read only)
```

### See Also

| File | Relationship |
|------|-------------|
| [feature/031_account_inspect.md](../../feature/031_account_inspect.md) | Unified account diagnostic — identity, subscription, org, and quota utilization |
| [feature/013_account_limits.md](../../feature/013_account_limits.md) | Rate-limit utilization (merged — inspect now includes quota data from endpoint 001) |
| [feature/017_token_refresh.md](../../feature/017_token_refresh.md) | Token refresh via isolated subprocess (when `refresh::1`) |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.inspect`](../command/001_account.md#command--15-accountinspect) | Live identity and subscription diagnostic |
