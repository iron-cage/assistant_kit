# Feature: Browser Re-Authentication for Named Account

### Scope

- **Purpose**: Re-authenticate a named account whose `refreshToken` is dead by spawning `claude` with an inherited TTY and capturing the resulting credentials.
- **Responsibility**: Documents the `.account.relogin` CLI command and its 6-step mechanism for interactive browser re-authentication.
- **In Scope**: TTY spawn, credential-change detection, store write-back, original-active restoration, dry-run mode, ownership guard (exit 1 when account is owned by a different identity — G7 gate from [036_account_ownership.md](036_account_ownership.md)).
- **Out of Scope**: Token refresh via isolated subprocess (→ [017_token_refresh.md](017_token_refresh.md)); browser login mechanics (delegated to `claude` binary).

### Design

When `refresh::1` silently fails (`run_isolated` returns `credentials=None`), the `refreshToken` itself is expired and cannot be renewed without user interaction. `.account.relogin` handles this recovery path by:

**Name resolution**: When `name::` is omitted, the active account (per-machine active marker) is used. If `name::` is omitted AND no per-machine active marker exists, the command exits 2 with an actionable message. This follows invariant [006_param_defaults.md](../invariant/006_param_defaults.md): parameters must default to active context when possible.

1. Resolving `name::` via AccountSelector (full email or prefix), or reading the per-machine active marker when `name::` is omitted → validate account exists.
2. Snapshotting the per-machine active marker (best-effort; `None` when absent).
3. Calling `switch_account(name)` to copy the named account's credentials into `~/.claude/.credentials.json` so `claude` picks up its `refreshToken`.
4. Spawning `claude` with **inherited TTY** (`stdin`/`stdout`/`stderr` connected) — the user completes browser re-authentication interactively.
5. After `claude` exits: comparing `~/.claude/.credentials.json` content before and after. If changed → call `account::save(name)` to write the refreshed credentials back to the store.
6. Restoring the original active account via `switch_account(original_active)` (best-effort; non-fatal on failure).

**NOT `run_isolated`** — this command requires an interactive TTY. Running it in a piped non-TTY context will cause Claude to fail at startup.

**Dry-run mode** (`dry::1`): Prints `[dry-run] would re-authenticate '{name}' via browser login` without executing any of the 6 steps.

**Ownership guard (G7):** Before executing step 1, `account_relogin_routine()` reads the `owner` field from `{name}.json`. If `owner` is non-empty and does not match `current_identity()`, the command exits 1 with `"ownership violation: this account is owned by {owner}"`. This check runs before `dry::1` output — a dry-run on a non-owned account still exits 1. See [036_account_ownership.md](036_account_ownership.md).

**Exit codes:**
- 0: success — credentials refreshed and saved, original active restored.
- 1: usage error — empty or invalid characters in `name::` value; or ownership violation (G7 gate).
- 2: runtime error — `name::` omitted and no active account; account not found; HOME unset; `claude` binary cannot be spawned; or `save()` fails after credential update.
- 3 (via `process::exit`): login abandoned — `claude` exited without updating `~/.claude/.credentials.json`. A diagnostic message must be printed to stderr before exiting.

### Acceptance Criteria

- **AC-01**: `clp .account.relogin name::carol@example.com dry::1` exits 0 with `[dry-run] would re-authenticate 'carol@example.com' via browser login`; no files mutated.
- **AC-02**: `clp .account.relogin` (no `name::`) with an active account uses the active account; `dry::1` outputs `[dry-run] would re-authenticate '{active}' via browser login`.
- **AC-03**: `clp .account.relogin` (no `name::`) with no active account exits 2 with an actionable message.
- **AC-04**: Non-existent account name exits 2 with "not found".
- **AC-05**: Positional bare arg `clp .account.relogin carol@example.com` is accepted (AccountSelector).
- **AC-06**: Prefix form `clp .account.relogin car` resolves to the single matching account.
- **AC-07**: After successful browser login, the named account's credential file in the store is updated (same as if `.account.save` had been run).
- **AC-08**: After re-authentication, the original active account is restored — the user's session context is unchanged.
- **AC-09**: If `claude` exits without updating `~/.claude/.credentials.json`, a diagnostic message is printed to stderr indicating credentials were unchanged, and the process exits 3.
- **AC-10**: `clp .account.relogin name::alice@corp.com` when `alice@corp.com.json` has `owner` ≠ `current_identity()` exits 1 with `"ownership violation: this account is owned by {owner}"`. No files are modified. (G7 ownership gate — [036_account_ownership.md](036_account_ownership.md) AC-10.)
- **AC-11**: Ownership check runs before `dry::1` output — `clp .account.relogin name::alice@corp.com dry::1` with ownership violation exits 1 without printing the dry-run message.

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command-12-accountrelogin) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [017_token_refresh.md](017_token_refresh.md) | Automated refresh path — use `.account.relogin` when `refresh::1` returns `credentials=None` |
| [036_account_ownership.md](036_account_ownership.md) | G7: ownership guard — exit 1 before any relogin steps when account is owned by different identity |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/006_param_defaults.md](../invariant/006_param_defaults.md) | Governing principle: `name::` defaults to active account when omitted |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.relogin`](../cli/command/001_account.md#command-12-accountrelogin) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/account_relogin.rs` | `account_relogin_routine()` — CLI handler; 6-step TTY spawn and credential capture |
| `src/account.rs` | `switch_account()`, `save()` — credential rotation and store write-back |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/cli/command/12_account_relogin.md](../../tests/docs/cli/command/12_account_relogin.md) | Integration test plan |

### Subprocess Docs

| File | Relationship |
|------|-------------|
| [subprocess/005_relogin_invocation.md](../subprocess/005_relogin_invocation.md) | Relogin vs. `run_isolated()` comparison; TTY inheritance; active account restore |
| [state_machine/002_oauth_token_lifecycle.md](../state_machine/002_oauth_token_lifecycle.md) | RT-expired state and relogin as recovery transition |
