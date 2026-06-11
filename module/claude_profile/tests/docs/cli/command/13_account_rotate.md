# Command :: 13. `.account.rotate` — Integration Tests

Auto-rotate to the best inactive account (highest remaining token expiry) with no name argument.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Empty credential store → exit 2 | No `*.credentials.json` files | 2 |
| IT-2 | Single account and it is active → exit 2 | Only one saved account, same as active | 2 |
| IT-3 | Rotates to best inactive account | Two accounts; active has lower expiry; checks per-machine active marker changed | 0 |
| IT-4 | Selects highest `expiresAt` among multiple inactives | Three accounts with differing expiry; verifies selected name | 0 |
| IT-5 | `dry::1` shows candidate without switching | Two accounts present; per-machine active marker unchanged after command | 0 |
| IT-6 | `dry::1` output contains `[dry-run]` prefix | dry::1 | 0 |
| IT-7 | Output line confirms rotated account name | Rotation succeeds; output contains `rotated to` + name | 0 |
| IT-8 | Unknown parameter rejected → exit 1 | `clp .account.rotate unknown::x` | 1 |
