# Subprocess: Browser Relogin Invocation

### Scope

- **Purpose**: Document how `.account.relogin` spawns `claude` with an inherited TTY to recover accounts whose OAuth refresh token has expired and cannot be recovered via isolated subprocess.
- **Responsibility**: Authoritative reference for the relogin mechanism, TTY inheritance, active account restore procedure, and comparison with `run_isolated()`.
- **In Scope**: Relogin mechanism steps; TTY inheritance; active account credential restore; key differences from `run_isolated()`; when relogin is the correct recovery path.
- **Out of Scope**: `run_isolated()` API contract (→ `subprocess/001`); credential write-back for batch refresh (→ `subprocess/002`); full `.account.relogin` acceptance criteria (→ `feature/019`).

### Purpose

Document how `.account.relogin` spawns `claude` with an inherited TTY to recover accounts whose OAuth refresh token has expired (unrecoverable via isolated subprocess).

### When to Use

`refresh_account_token()` returns `None` when the refresh token is expired — Claude CLI rejects the RT with the OAuth server. In this case, the RT must be replaced via a browser-based OAuth flow. `.account.relogin` is the operational remedy.

### Mechanism

Unlike `run_isolated()` (which uses an isolated temp `HOME`), relogin:
1. Temporarily copies target account's credentials to `~/.claude/.credentials.json`
2. Spawns `claude` **with the real TTY inherited** (not piped) — this allows the browser OAuth flow to complete interactively
3. After `claude` exits, reads the updated `~/.claude/.credentials.json`
4. Writes new credentials back to `{name}.credentials.json` in the credential store
5. Restores the previously-active account's credentials to `~/.claude/.credentials.json`

### Key Differences from `run_isolated()`

| Property | `run_isolated()` | Relogin |
|----------|-----------------|---------|
| HOME | Isolated temp dir | Real `$HOME` |
| TTY | Piped (non-interactive) | Inherited (interactive) |
| Timeout | 35s | None (user-paced) |
| Purpose | Credential refresh (RT valid) | Browser OAuth (RT expired) |
| Writes to `~/.claude/.credentials.json` | Never (BUG-221) | Yes (temporarily, then restores) |

### Active Account Restore

After relogin completes, the previously-active account's credentials are restored to `~/.claude/.credentials.json` so the user's live session is undisturbed. Relogin targets a named account (not the active one).

### Features

| File | Relationship |
|------|-------------|
| [feature/019_account_relogin.md](../feature/019_account_relogin.md) | Full feature spec |

### Subprocess

| File | Relationship |
|------|-------------|
| [subprocess/001](001_run_isolated_contract.md) | `run_isolated()` — different mechanism |
| [subprocess/003](003_token_refresh_invocation.md) | Token refresh (prerequisite attempt before relogin) |
