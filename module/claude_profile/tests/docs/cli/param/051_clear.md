# Test: `clear::` Parameter

Edge case coverage for the `clear::` parameter on `.account.renewal`. See [param/051_clear.md](../../../../docs/cli/param/051_clear.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clear::1` removes `_renewal_at` from `{name}.claude.json` | Behavioral: remove |
| EC-2 | `clear::1` on account without `_renewal_at` exits 0 (no-op) | Idempotency |
| EC-3 | `clear::1` preserves existing `oauthAccount` content | Preservation |
| EC-4 | After `clear::1`, `.usage` shows `~`-prefixed estimate again | Effect on display |
| EC-5 | `clear::` combined with `at::` exits 1 | Mutual Exclusion |
| EC-6 | `clear::` combined with `from_now::` exits 1 | Mutual Exclusion |

---

### EC-1: `clear::1` removes `_renewal_at` from `{name}.claude.json`

- **Given:** Account `test@example.com` has `_renewal_at: "2026-06-29T21:00:00Z"` in its `.claude.json`.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `{credential_store}/test@example.com.claude.json` no longer contains `_renewal_at` key.
- **Exit:** 0
- **Source fn:** `ft04_account_renewal_clear_removes_key` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-2: `clear::1` on account without `_renewal_at` exits 0 (no-op)

- **Given:** Account `test@example.com` exists but has no `_renewal_at` in its `.claude.json` (or no `.claude.json` at all).
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. No error. File state unchanged (or empty `{}` if `.claude.json` was absent).
- **Exit:** 0
- **Source fn:** `arn19_clear_no_prior_renewal_at_exits_0` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-3: `clear::1` preserves existing `oauthAccount` content

- **Given:** Account `test@example.com` has both `oauthAccount` and `_renewal_at` in `.claude.json`.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `_renewal_at` is absent. `oauthAccount` content is unchanged (read-merge preserved non-`_renewal_at` keys).
- **Exit:** 0
- **Source fn:** `arc02_clear_preserves_oauth_account_content` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-4: After `clear::1`, `.usage` shows `~`-prefixed estimate again

- **Given:** Account was saved with `_renewal_at` set (`.usage` showed exact `in Xh Ym`). `clear::1` applied.
- **When:** `clp .usage` after the clear.
- **Then:** `~Renews` column for that account shows `~in Xd` (estimated, with `~` prefix from `org_created_at`) — not the exact `in Xh Ym` format.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it237_lim_it_clear_usage_shows_tilde_estimate` (in `tests/cli/usage_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-5: `clear::` combined with `at::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com clear::1 at::2026-06-29T21:00:00Z`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft08_account_renewal_at_clear_conflict` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-6: `clear::` combined with `from_now::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com clear::1 from_now::+1h`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft09_account_renewal_from_now_clear_conflict` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)
