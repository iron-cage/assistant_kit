# Feature: Account Name Shortcut Syntax

### Scope

- **Purpose**: Allow `name::` to be supplied as a bare positional argument or resolved from a prefix, removing friction when typing account-management commands.
- **Responsibility**: Documents the positional-argument adapter rewrite and prefix-resolution logic for the four `name::` commands.
- **In Scope**: Positional rewrite in `src/adapter.rs`; prefix resolution in `src/commands.rs` affecting `.accounts`, `.account.use`, `.account.delete`, `.account.limits`.
- **Out of Scope**: Email validation (`account::validate_name()` — unchanged); `.account.save` name inference from `~/.claude.json` (→ 002_account_save.md); `~/.claude/.credentials.json` live account detection (→ 009_token_usage.md).

### Design

Two complementary shortcuts reduce typing without changing the underlying `name::` parameter:

**Positional argument (adapter layer):**
When a command that accepts `name::` receives a bare token (no `::`) as its first parameter, the adapter layer (`argv_to_unilang_tokens()`) rewrites it to `name::{value}` before the unilang pipeline. The following commands participate: `.accounts`, `.account.use`, `.account.delete`, `.account.limits`.

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

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/adapter.rs` | `argv_to_unilang_tokens()` — positional rewrite for name-taking commands |
| source | `src/commands.rs` | `account_use_routine`, `account_delete_routine`, `accounts_routine`, `account_limits_routine` — prefix resolution |
| source | `src/lib.rs` | `cli::print_usage()` — update example to use positional form |
| test | `tests/cli/account_mutations_test.rs` | account.use (aw13–aw15) and account.delete (ad13–ad14) positional and prefix cases |
| test | `tests/cli/accounts_test.rs` | accounts (acc29–acc30) positional and prefix cases |
| test | `tests/cli/account_limits_test.rs` | account.limits (lim09–lim10) positional and prefix cases |
| doc | [cli/param/001_name.md](../cli/param/001_name.md) | `name::` parameter specification |
| doc | [command/readme.md](../cli/command/readme.md) | Syntax blocks for affected commands |
| doc | [004_account_use.md](004_account_use.md) | Base switch behavior |
| doc | [005_account_delete.md](005_account_delete.md) | Base delete behavior |
