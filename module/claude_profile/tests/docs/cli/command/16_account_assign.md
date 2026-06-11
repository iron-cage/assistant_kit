# Command :: 16. `.account.assign` — Integration Tests

Write the per-machine active-account marker for any host+user pair without credential rotation. Marker-only write — no `~/.claude.*` side effects.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Default (no `for::`) writes current-machine marker | `name::alice@corp.com`; no `for::`; `_active_{hostname}_{user}` created = `alice@corp.com` | 0 |
| IT-2 | `for::bob@laptop` writes `_active_laptop_bob` | Account exists; `for::bob@laptop`; `_active_laptop_bob` = account name in credential store | 0 |
| IT-3 | Dry-run prints `[dry-run]` line; no file written | `dry::1`; no `_active_*` file present afterward | 0 |
| IT-4 | No `name::` emits live usage block | No arguments; stdout contains `Current machine:` and `Ready to copy:` | 0 |
| IT-5 | Unknown account name exits 2 | `name::nobody@example.com` not in credential store | 2 |
| IT-6 | `for::` without `@` exits 1 | `for::badvalue` — no `@` separator | 1 |
| IT-7 | Empty `for::` component exits 1 | `for::@laptop` (empty user) or `for::bob@` (empty machine) | 1 |
| IT-8 | `~/.claude/.credentials.json` unchanged after assign | Run `.account.assign`; verify mtime of credentials file unchanged | 0 |
| IT-9 | Prefix resolution via `name::alice` | `alice@corp.com` only account with prefix `alice`; `_active_*` = `alice@corp.com` | 0 |
| IT-10 | Overwrite existing marker | `_active_laptop_bob` exists with old account; command writes new account name | 0 |
| IT-11 | Unknown parameter rejected | `clp .account.assign unknown::x` | 1 |
| IT-12 | Dry-run with `for::` shows target marker filename | `for::bob@laptop dry::1`; stdout contains `_active_laptop_bob` | 0 |
