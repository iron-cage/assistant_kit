# Feature: Auto Rotate

> **DEPRECATED** — Removed in favor of `.usage rotate::1`. The `auto_rotate()` API and `.account.rotate` CLI command have been dropped. Use `clp .usage rotate::1` (with optional `sort::` strategy) to execute strategy-driven account rotation. See [feature/038_usage_strategy_rotate.md](038_usage_strategy_rotate.md).

### Scope

- **Purpose**: Enable one-call account rotation for automation that needs to swap accounts without writing selection logic.
- **Responsibility**: Documents the `account::auto_rotate()` API (FR-13) and its CLI surface (`.account.rotate`, Command 13).
- **In Scope**: Best-inactive-account selection (highest `expires_at_ms`), single-call rotation, CLI dry-run preview.
- **Out of Scope**: Custom selection strategies (caller implements those using `list()` + `switch_account()`), token expiry detection (→ 006_token_status.md).

### Design

`claude_profile` must provide `account::auto_rotate( credential_store, paths )` as a one-call rotation primitive:

1. Call `account::list( credential_store )` to enumerate all accounts.
2. Filter to inactive accounts (`is_active == false`).
3. Select the account with the highest `expires_at_ms`.
4. Call `switch_account( name, credential_store, paths )` on the selected account.
5. Return the name of the account switched to.

**Not-found conditions — both return `NotFound`:**
- No accounts configured at all.
- All accounts are active (only one account exists and it is the current one).

**Decomposed rotation** (for custom selection logic):

```
1. Detect token state:    token::status() → TokenStatus::Expired
2. List candidates:       account::list( credential_store ) → Vec<Account>
3. Select best:           filter is_active==false, pick max expires_at_ms
4. Switch:                account::switch_account(name, credential_store, paths) → writes credentials + active marker
5. Confirm (caller):      invoke claude_runner with minimal message to verify
```

### Acceptance Criteria

- **AC-01**: `auto_rotate( credential_store, paths )` returns the name of the inactive account with the highest `expires_at_ms`.
- **AC-02**: `auto_rotate( credential_store, paths )` returns `NotFound` when no inactive accounts exist.
- **AC-03**: After `auto_rotate( credential_store, paths )`, `~/.claude/.credentials.json` contains the selected account's credentials.

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md#command--13-accountrotate](../cli/command/001_account.md#command--13-accountrotate) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [004_account_use.md](004_account_use.md) | Switch primitive used by auto_rotate |
| [006_token_status.md](006_token_status.md) | Token detection to trigger rotation |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Configurable sort strategies for `.usage`; references this command's selection algorithm as a contrast point |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.rotate`](../cli/command/001_account.md#command--13-accountrotate) | CLI surface for this feature |

### Sources

> Sources removed — `auto_rotate()` deleted from `src/account.rs` and `account_rotate_routine()` deleted from `src/commands/account_ops.rs` as part of Feature 038 migration.

### Tests

| File | Relationship |
|------|--------------|
| `tests/account_tests.rs::auto_rotate_*` | Rotation selection and switch tests |
| [tests/docs/cli/command/013_account_rotate.md](../../tests/docs/cli/command/013_account_rotate.md) | CLI integration test spec (IT-1..IT-8) |
