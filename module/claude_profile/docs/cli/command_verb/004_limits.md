# Verb: limits

Reads current rate-limit utilization for an account by making a lightweight HTTP request to the Claude API and inspecting `anthropic-ratelimit-unified-*` response headers. Returns the current usage and ceiling for the rate-limited quota window. No local state is written.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.limits` | Yes | No |

### Behavioral Contract

**Pre-conditions:**
- Named account (or active account if `name::` omitted) credentials accessible in credential store
- Network reachable; Claude API endpoint accessible
- `$HOME` environment variable set

**Post-conditions:**
- Rate-limit utilization reported from live API headers
- No local files written or modified

**Side effects:**
- Makes one HTTP request to Claude API; does not consume meaningful quota (lightweight probe)
- Response headers parsed for `anthropic-ratelimit-unified-*` fields

### Idempotency

**Yes.** Pure read — fetches live API response headers. Repeated calls return the current limit state at call time; no side effects accumulate.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to check; defaults to active account | No |
| `format::` | Output format (`text` or `json`) | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Reads state.** Makes HTTP request to Claude API; reads `anthropic-ratelimit-unified-*` response headers. No local writes. The account lifecycle state is unchanged.

```
[saved/active] --account.limits--> [saved/active]  (state unchanged; read only)
```

### See Also

| File | Relationship |
|------|-------------|
| [feature/013_account_limits.md](../../feature/013_account_limits.md) | Rate-limit header parsing and utilization display |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Show rate-limit utilization for named account |
