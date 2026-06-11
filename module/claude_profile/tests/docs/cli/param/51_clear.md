# Test: `clear::` Parameter

Edge case coverage for the `clear::` parameter on `.account.renewal`. See [param/051_clear.md](../../../../docs/cli/param/051_clear.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clear::1` removes `_renewal_at` from per-account `{name}.json` | Behavioral: remove |
| EC-2 | `clear::1` on account without `_renewal_at` exits 0 (no-op) | Idempotency |
| EC-3 | `clear::1` preserves existing `oauthAccount` content | Preservation |
| EC-4 | After `clear::1`, `.usage` shows `~`-prefixed estimate again | Effect on display |
| EC-5 | `clear::` combined with `at::` exits 1 | Mutual Exclusion |
| EC-6 | `clear::` combined with `from_now::` exits 1 | Mutual Exclusion |
| EC-7 | `clear::1` removes `_renewal_at`; subsequent `.usage` shows `~` estimate | Behavioral Divergence |
| EC-8 | `.account.renewal` without `clear::` — `_renewal_at` unchanged | Behavioral Divergence |

---

### EC-1: `clear::1` removes `_renewal_at` from per-account `{name}.json`

- **Given:** Account `test@example.com` has `_renewal_at: "2026-06-29T21:00:00Z"` in its `{name}.json`.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` no longer contains `_renewal_at` key.
- **Exit:** 0
- **Source fn:** `ft04_account_renewal_clear_removes_key` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-2: `clear::1` on account without `_renewal_at` exits 0 (no-op)

- **Given:** Account `test@example.com` exists but has no `_renewal_at` in its `{name}.json` (or no `{name}.json` at all).
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. No error. File state unchanged (or empty `{}` if `{name}.json` was absent).
- **Exit:** 0
- **Source fn:** `arn19_clear_no_prior_renewal_at_exits_0` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-3: `clear::1` preserves existing `oauthAccount` content

- **Given:** Account `test@example.com` has both `oauthAccount` and `_renewal_at` in `{name}.json`.
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

---

### EC-7: `clear::1` removes `_renewal_at`; subsequent `.usage` shows `~` estimate

- **Behavioral Divergence:** Running `.account.renewal clear::1` removes the `_renewal_at` key. A following `.usage` command shows a `~`-prefixed estimated date in `~Renews`. Running `.account.renewal` without `clear::` (EC-8) leaves `_renewal_at` intact, so `.usage` continues to show the exact countdown.
- **Given:** Account `test@example.com` has `_renewal_at: "2028-01-01T00:00:00Z"` set.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `_renewal_at` is absent from `{name}.json`. A subsequent `clp .usage` shows `~` prefix in the `~Renews` column for this account.
- **Exit:** 0
- **Source fn:** `arc03_clear_removes_key_usage_shows_tilde` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-8: `.account.renewal` without `clear::` leaves `_renewal_at` unchanged

- **Behavioral Divergence:** (pair with EC-7)
- **Given:** Account `test@example.com` has `_renewal_at: "2028-01-01T00:00:00Z"` set.
- **When:** `clp .account.renewal name::test@example.com` (no `at::`, `from_now::`, or `clear::`)
- **Then:** Exits 1 (no action parameter supplied — renewal command requires one of `at::`, `from_now::`, or `clear::1`). `_renewal_at` remains unchanged in `{name}.json`.
- **Exit:** 1
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)
