# Verb: relogin

Forces browser re-authentication for an account whose OAuth refresh token has expired or become invalid. Executes a TTY subprocess sequence: switch to target account, spawn the `claude` binary (which opens a browser OAuth flow), detect the resulting credential change, save the new credentials back to the named profile, then restore the prior session.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.relogin` | No | No |

### Behavioral Contract

**Pre-conditions:**
- Named account (or active account if `name::` omitted) profile exists in credential store
- `claude` binary available on `$PATH`
- TTY available for browser OAuth redirect interaction
- `$HOME` environment variable set

**Post-conditions:**
- Account credentials refreshed in-place with new OAuth tokens
- Named account profile updated with fresh credentials
- Prior active session restored (if target was not already active)
- Account lifecycle state unchanged (`saved` remains `saved`, `active` remains `active`)

**Side effects:**
- Opens browser for OAuth login flow
- Temporarily switches active session to target account (restored after completion)
- Spawns `claude` subprocess for credential capture
- Writes new credentials to `{name}.credentials.json` via `.account.save`

### Idempotency

**No.** Each invocation initiates a browser OAuth flow and generates new tokens. Not safe to repeat without user intent; each run may produce different tokens even for the same account.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to re-authenticate; defaults to active account | No |
| `force::` | Bypass G7 ownership gate; allow re-authenticating a non-owned account | No |
| `dry::` | Validate without executing browser flow | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Transitions state.** Updates credentials in-place for the named account via switch → spawn → detect → save → restore sequence. Lifecycle state (saved/active) is preserved.

```
[saved]  --account.relogin--> [saved]   (credentials refreshed; state unchanged)
[active] --account.relogin--> [active]  (credentials refreshed; state unchanged)
```

### See Also

| File | Relationship |
|------|-------------|
| [feature/019_account_relogin.md](../../feature/019_account_relogin.md) | TTY subprocess sequence and credential change detection |
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | G7 ownership gate — exits 1 when account owned by different identity; `force::1` bypasses G7 |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Force browser re-authentication for named account |
