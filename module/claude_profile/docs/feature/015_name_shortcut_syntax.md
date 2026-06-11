# Feature: Account Name Shortcut Syntax

### Scope

- **Purpose**: Allow `name::` to be supplied as a bare positional argument or resolved from a prefix, removing friction when typing account-management commands.
- **Responsibility**: Documents the positional-argument adapter rewrite and prefix-resolution logic for `name::` commands.
- **In Scope**: Positional rewrite in `src/adapter.rs`; prefix resolution in `src/commands/accounts.rs`, `src/commands/account_ops.rs`, `src/commands/account_relogin.rs`, `src/commands/account_renewal.rs` affecting `.accounts`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, and `.account.renewal` (comma-list token resolution).
- **Out of Scope**: Email validation (`account::validate_name()` — unchanged); `.account.save` name inference from `~/.claude.json` (→ 002_account_save.md); `~/.claude/.credentials.json` live account detection (→ 009_token_usage.md).

### Design

Two complementary shortcuts reduce typing without changing the underlying `name::` parameter:

**Positional argument (adapter layer):**
When a command that accepts `name::` receives a bare token (no `::`) as its first parameter, the adapter layer (`argv_to_unilang_tokens()`) rewrites it to `name::{value}` before the unilang pipeline. The following commands participate: `.accounts`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`.

```bash
clp .account.use alice@home.com       # rewritten to: .account.use name::alice@home.com
clp .account.delete alice@oldco.com   # rewritten to: .account.delete name::alice@oldco.com
```

The `name::` form continues to work unchanged — positional and explicit forms are equivalent.

**Prefix resolution (command layer):**
When the `name` value contains no `@` character, the command resolves it as a prefix against saved account names. Resolution algorithm:
1. Sort all saved account names alphabetically.
2. Check for an exact local-part match: if exactly one account's local part (the portion before `@`) equals the prefix string exactly, resolve to that account immediately. This prevents `i1` from being ambiguous when `i1@host`, `i11@host`, and `i12@host` all exist.
3. Find all names that start with the given prefix string.
4. Exactly one match → use the resolved full name (proceed as with an explicit `name::EMAIL`).
5. Zero matches → exit 2 with "account not found: '{prefix}'".
6. Two or more matches → exit 1 with "ambiguous prefix '{prefix}': matches {A}, {B}, ..." (up to first 3 shown).

Prefix resolution applies AFTER positional rewriting: `clp .account.use car` → adapter rewrites to `name::car` → command resolves `car` → `carol@example.com`.

**Email detection heuristic:** A value containing `@` is treated as a full email address (no prefix resolution). The existing `account::validate_name()` email validation applies to the final resolved name.

### Acceptance Criteria

- **AC-01**: `clp .account.use alice@home.com` exits 0 and switches to `alice@home.com` — identical to `clp .account.use name::alice@home.com`.
- **AC-02**: `clp .account.delete alice@oldco.com` exits 0 and deletes the account — identical to `clp .account.delete name::alice@oldco.com`.
- **AC-03**: `clp .accounts alice@home.com` exits 0 and shows one indented block — identical to `clp .accounts name::alice@home.com`.
- **AC-04**: `clp .account.limits alice@acme.com` exits 0 and shows limits — identical to `clp .account.limits name::alice@acme.com`.
- **AC-05**: `clp .account.use car` (where `carol@example.com` is saved) resolves to `carol@example.com` and exits 0.
- **AC-06**: `clp .account.use a` (where `alice@example.com` and `amy@example.com` are saved) exits 1 with an ambiguous-prefix message listing the matches.
- **AC-07**: `clp .account.use ghost` (no account starts with `ghost`) exits 2 with a not-found error.
- **AC-08**: Existing `name::EMAIL` explicit form continues to work unchanged on all four commands.
- **AC-09**: `clp .account.use alice@home.com dry::1` works — positional and `dry::` can be combined.
- **AC-10**: The `print_usage()` Examples section shows `clp .account.use alice@acme.com` (without `name::` prefix).
- **AC-11**: `clp .account.use i1` where `i1@wbox.pro`, `i11@wbox.pro`, and `i12@wbox.pro` all exist → exits 0 and switches to `i1@wbox.pro` (exact local-part match wins over longer prefix matches).
- **AC-12**: `clp .account.renewal name::alice at::2026-07-01T00:00:00Z` (where `alice@acme.com` is the only saved account whose local part is `alice`) → resolves to `alice@acme.com`, writes `_renewal_at`, exits 0.
- **AC-13**: `clp .account.renewal name::alice,bob at::2026-07-01T00:00:00Z` → resolves each comma token independently via prefix resolution; `alice@acme.com` and `bob@acme.com` both updated; exits 0.

### Commands

| File | Relationship |
|------|--------------|
| [command/readme.md](../cli/command/readme.md) | Syntax blocks for affected commands |

### Features

| File | Relationship |
|------|--------------|
| [004_account_use.md](004_account_use.md) | Base switch behavior |
| [005_account_delete.md](005_account_delete.md) | Base delete behavior |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Per-machine active marker naming convention; AC-11 added for exact-local-part match |
| [030_account_renewal_override.md](030_account_renewal_override.md) | `.account.renewal` multi-account dispatch and `name::all`/comma-list handling |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` parameter specification |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [Command Index](../cli/command/readme.md) | Syntax blocks for all name-taking commands |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapter.rs` | `argv_to_unilang_tokens()` — positional rewrite for name-taking commands |
| `src/commands/account_ops.rs` | `account_use_routine`, `account_delete_routine` — prefix resolution |
| `src/commands/account_relogin.rs` | `account_relogin_routine` — prefix resolution |
| `src/commands/account_renewal.rs` | `account_renewal_routine` — comma-list token resolution |
| `src/commands/accounts.rs`, `src/commands/limits.rs` | `accounts_routine`, `account_limits_routine` — prefix resolution |
| `src/lib.rs` | `cli::print_usage()` — update example to use positional form |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_mutations_test.rs` | account.use (aw13–aw15), account.delete (ad13–ad14), and account.renewal (ar15–ar16) positional, prefix, and comma-list cases |
| `tests/cli/accounts_test.rs` | accounts (acc29–acc30) positional and prefix cases |
| `tests/cli/account_limits_test.rs` | account.limits (lim09–lim10) positional and prefix cases |
