# Command :: 14. `.account.renewal` — Integration Tests

Set or clear a billing renewal timestamp override (`_renewal_at`) stored in `{name}.json`.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | `at::` writes `_renewal_at` to `{name}.json` | Account exists; `at::2026-06-29T21:00:00Z`; verifies file contains field | 0 |
| IT-2 | `from_now::+1h` writes future `_renewal_at` | Account exists; `from_now::+1h`; verifies ISO-8601 string is present | 0 |
| IT-3 | `clear::1` removes `_renewal_at` from `{name}.json` | Account has `_renewal_at`; after `clear::1`, field absent | 0 |
| IT-4 | `at::` and `from_now::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-5 | `at::` and `clear::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-6 | `from_now::` and `clear::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-7 | No operation param provided exits 1 | `name::` only, no `at::`/`from_now::`/`clear::` | 1 |
| IT-8 | `dry::1` prints would-be value without writing | `at::2026-06-29T21:00:00Z dry::1`; file unchanged after command | 0 |
| IT-9 | `name::all` updates every saved account | Two accounts; `from_now::+0m`; both `.json` updated | 0 |
| IT-10 | `name::` with unknown account exits 2 | Account not in credential store | 2 |
| IT-11 | `name::alice,bob` comma-list updates both | Two accounts; `at::2026-06-29T21:00:00Z`; both updated | 0 |
| IT-12 | Existing `oauthAccount` content preserved on write | Account with full `oauthAccount`; after `at::`; `oauthAccount` unchanged | 0 |
| IT-13 | Past `at::` value accepted (auto-advanced by `.usage`) | `at::2020-01-01T00:00:00Z`; exits 0; field written verbatim | 0 |
| IT-14 | Unknown parameter rejected → exit 1 | `clp .account.renewal unknown::x` | 1 |
| IT-15 | Single prefix resolves to full email | Account `alice@acme.com` saved; `name::alice at::2026-07-01T00:00:00Z`; `_renewal_at` written to `alice@acme.com.json` | 0 |
| IT-16 | Comma-list with prefix tokens resolves each | Accounts `alice@acme.com` and `bob@acme.com` saved; `name::alice,bob at::2026-07-01T00:00:00Z`; both `.json` files updated | 0 |
