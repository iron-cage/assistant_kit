# Command :: 18. `.account.unclaim` — Integration Tests

Release account ownership by clearing the `owner` field in `{name}.json` via `write_owner()` directly. No credentials touched. No active marker changed. G8 ownership gate enforced before write.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Clears `owner` field — caller is current owner | `name::alice@acme.com`; `alice.json` has `"owner": "testuser@testmachine"`; current identity = `testuser@testmachine`; `alice.json` has `"owner": ""` after | 0 |
| IT-2 | Credential file NOT touched after unclaim | `name::alice@acme.com`; mtime of `alice.credentials.json` identical before and after | 0 |
| IT-3 | Active marker NOT changed after unclaim | `name::alice@acme.com`; `_active_{hostname}_{user}` unchanged after command | 0 |
| IT-4 | Already-unowned account is idempotent — exits 0 | `name::alice@acme.com`; `alice.json` has `"owner": ""`; exits 0; `alice.json` retains `"owner": ""` | 0 |
| IT-5 | G8 gate — exits 1 when account owned by different identity | `name::alice@acme.com`; `alice.json` has `"owner": "other@remote"`; current identity ≠ `other@remote`; stderr contains `ownership violation` | 1 |
| IT-6 | Dry-run prints intent; no file written | `name::alice@acme.com dry::1`; stdout contains `[dry-run] would unclaim alice@acme.com`; `alice.json` unchanged | 0 |
| IT-7 | G8 gate evaluates before dry-run | `name::alice@acme.com dry::1`; `alice.json` has `"owner": "other@remote"`; exits 1 with ownership violation; dry-run line NOT printed | 1 |
| IT-8 | Unknown account name exits 2 | `name::nobody@example.com` not in credential store | 2 |
| IT-9 | `name::` required — missing exits 1 | No `name::` argument; exits 1 with error indicating `name::` is required | 1 |
| IT-10 | Unknown parameter rejected | `clp .account.unclaim name::alice unknown::x` | 1 |
| IT-11 | Preserves non-ownership `{name}.json` fields via read-merge | `alice.json` has `_renewal_at: "2026-06-29T21:00:00Z"` and `"owner": "testuser@testmachine"`; after unclaim: `_renewal_at` retained, `owner: ""` | 0 |
