# Parameter :: 50. `from_now::`

Sets the billing renewal override timestamp to a computed offset from the current time. Computes `now ± delta` at execution time and stores the result as ISO-8601 UTC in `_renewal_at` in `{name}.json`.

- **Default:** *(omit; required when neither `at::` nor `clear::1` is provided)*
- **Constraints:** Must begin with `+` or `-`; supports `d` (days), `h` (hours), `m` (minutes); units may be combined (`+1d12h`); `+0m` means the current moment
- **Mutually exclusive with:** `at::`, `clear::`
- **Purpose:** Set the billing renewal override relative to now without knowing the exact ISO-8601 timestamp. Useful when the renewal just happened (`-30m`), is imminent (`+3h30m`), or when bulk-setting all accounts to start monthly auto-advance cycles (`+0m`).

**Delta format:**
- `+3h30m` → current time + 3 hours 30 minutes
- `-30m` → current time − 30 minutes (renewal just passed)
- `+1d` → current time + 1 day
- `+0m` → current time exactly (immediately enters monthly auto-advance)
- `+1d12h` → combined: 1 day and 12 hours

**Usage:**

```bash
clp .account.renewal name::alice@acme.com from_now::+3h30m
clp .account.renewal name::alice@acme.com from_now::-30m
clp .account.renewal name::all from_now::+0m
clp .account.renewal name::alice@acme.com from_now::+1d dry::1
```

**See Also:** [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full semantics and auto-advance behavior.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.renewal`](../command/001_account.md#command--14-accountrenewal) | Set renewal timestamp as offset from current time |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Quick relative renewal date entry during account setup |
