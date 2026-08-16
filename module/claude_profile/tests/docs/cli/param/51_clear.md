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
- **Source fn:** `ft04_account_renewal_clear_removes_key` (in `account_renewal_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-2: `clear::1` on account without `_renewal_at` exits 0 (no-op)

- **Given:** Account `test@example.com` exists but has no `_renewal_at` in its `{name}.json` (or no `{name}.json` at all).
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. No error. File state unchanged (or empty `{}` if `{name}.json` was absent).
- **Exit:** 0
- **Source fn:** `arn19_clear_no_prior_renewal_at_exits_0` (in `account_renewal_test_b.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-3: `clear::1` preserves existing `oauthAccount` content

- **Given:** Account `test@example.com` has both `oauthAccount` and `_renewal_at` in `{name}.json`.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `_renewal_at` is absent. `oauthAccount` content is unchanged (read-merge preserved non-`_renewal_at` keys).
- **Exit:** 0
- **Source fn:** `arc02_clear_preserves_oauth_account_content` (in `account_renewal_test_b.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-4: After `clear::1`, `.usage` exits 0 — no assertion on `~Renews` column content

> **Semantic drift correction:** the cited test's final `.usage` call asserts only `exit == 0` — it does not inspect the `~Renews` column, the `~` prefix, or any other display content. The function's own name (`..._shows_tilde_estimate`) and this doc's original claim describe a display verification the body does not implement. The test's real, verified assertion is that after `clear::1`, `_renewal_at` is absent from the account's JSON file (a genuine, correctly-cited check) — the *display* consequence of that removal (tilde-estimate appearing in `.usage`) is asserted by name only, not by content.

- **Given:** Account was saved with `_renewal_at` set. `clear::1` applied.
- **When:** `clp .usage` after the clear.
- **Then:** Exits 0. (The test verifies `_renewal_at` is absent from the JSON file after the clear — a real, correctly-cited assertion. It does NOT verify the `~Renews` column's content, the `~` prefix, or any `in Xd`-style estimate text; the final `.usage` call is exit-code-only.)
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it237_lim_it_clear_usage_shows_tilde_estimate` (in `usage_lim_it_test_b.rs`) — name and doc claim describe a tilde-estimate display verification the body does not implement; the final `.usage` call is exit-code-only
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-5: `clear::` combined with `at::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com clear::1 at::2026-06-29T21:00:00Z`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft08_account_renewal_at_clear_conflict` (in `account_renewal_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-6: `clear::` combined with `from_now::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com clear::1 from_now::+1h`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft09_account_renewal_from_now_clear_conflict` (in `account_renewal_test.rs`)
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-7: `clear::1` removes `_renewal_at`; subsequent `.usage` shows `~` estimate

- **Behavioral Divergence:** Running `.account.renewal clear::1` removes the `_renewal_at` key. A following `.usage` command shows a `~`-prefixed estimated date in `~Renews`. Running `.account.renewal` without `clear::` (EC-8) leaves `_renewal_at` intact, so `.usage` continues to show the exact countdown.
- **Given:** Account `test@example.com` has `_renewal_at: "2028-01-01T00:00:00Z"` set.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `_renewal_at` is absent from `{name}.json`. A subsequent `clp .usage` shows `~` prefix in the `~Renews` column for this account.
- **Exit:** 0
- **Source fn:** *(coverage gap — no `arc03`-named function exists anywhere in the suite; `account_mutations_test.rs` has no such function. EC-4's `it237_lim_it_clear_usage_shows_tilde_estimate` (live-tagged, in `usage_lim_it_test_b.rs`) exercises the clear-then-`.usage` sequence but — per EC-4's own correction above — its final `.usage` call is exit-code-only and does not verify tilde-estimate display content either; no test in the suite verifies the `~Renews` column's actual content after a `clear::1`)*
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### EC-8: `.account.renewal` without `clear::` leaves `_renewal_at` unchanged

- **Behavioral Divergence:** (pair with EC-7)
- **Given:** Account `test@example.com` has `_renewal_at: "2028-01-01T00:00:00Z"` set.
- **When:** `clp .account.renewal name::test@example.com` (no `at::`, `from_now::`, or `clear::`)
- **Then:** Exits 1 (no action parameter supplied — renewal command requires one of `at::`, `from_now::`, or `clear::1`). `_renewal_at` remains unchanged in `{name}.json`.
- **Exit:** 1
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)
