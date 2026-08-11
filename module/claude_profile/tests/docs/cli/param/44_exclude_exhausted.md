# Test: `exclude_exhausted::` Parameter

Edge case coverage for the `exclude_exhausted::` parameter on `.usage`. See [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `exclude_exhausted::1` shows only 🟢 rows | Behavioral Divergence |
| EC-2 | `exclude_exhausted::0` (default) shows all rows | Behavioral Divergence |
| EC-3 | `exclude_exhausted::1` is stricter than `only_valid::1` (also removes 🟡) | Comparison |
| EC-4 | `exclude_exhausted::bad` exits 1 naming valid values | Invalid Value |
| EC-5 | `exclude_exhausted::1` with all 🔴 accounts shows 0 rows | Empty Result |
| EC-6 | `exclude_exhausted::true` accepted (alias for 1) | Alias Acceptance |
| EC-7 | Cancelled account (`billing_type="none"`) hidden by `exclude_exhausted::1` | Cancelled Subscription |

---

### EC-1: `exclude_exhausted::1` shows only 🟢 rows (two-account fixture — no 🟡 account tested)

> **Semantic drift correction:** the cited test constructs only TWO accounts — one 🟢 (`live-acct@test.com`) and one 🔴 (`error-acct@test.com`) — not three, and no 🟡 account is present. Per the test's own doc comment, the 🟡 divergence "requires a real exhausted account state unavailable with shared tokens." Only the 🟢-shown/🔴-hidden half of the original claim is verified; the 🟡-hidden half is untested by this function.

- **Given:** Two accounts: one 🟢 (`live-acct@test.com`), one 🔴 (`error-acct@test.com`, invalid token).
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Only 🟢 row shown; 🔴 row hidden. (The 🟡-hidden half of the original claim is untested by this function — no 🟡 account is constructed; per the test's own doc comment, a real exhausted-quota account state is unavailable with shared tokens.)
- **Exit:** 0
- **Source fn:** `it229_lim_it_exclude_exhausted_1_shows_green` (in `usage_lim_it_test_b.rs`) — fixture is 🟢+🔴 only; doc's "three accounts including 🟡" claim not supported by this test's body
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-2: `exclude_exhausted::0` shows all rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::0`
- **Then:** Exits 0. All rows shown.
- **Exit:** 0
- **Source fn:** `it174_exclude_exhausted_0_shows_all_rows` (in `usage_filter_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-3: `exclude_exhausted::1` is at-least-as-strict as `only_valid::1` (🟢+🔴 fixture — not a 🟡 divergence test)

> **Semantic drift correction:** the cited test does not construct a 🟡 account and does not compare exact row sets between the two filters. It uses a 🟢+🔴 fixture (`live-acct@test.com`, `error-acct@test.com`) and verifies two weaker properties: (1) the row count under `exclude_exhausted::1` is less-than-or-equal-to the row count under `only_valid::1` (`rows_excl <= rows_valid`), and (2) the 🔴 account is explicitly asserted hidden from the `exclude_exhausted::1` output specifically — its absence from the `only_valid::1` output is inferred only via the row-count relationship, not asserted directly. It does not demonstrate the specific "exclude_exhausted removes 🟡 while only_valid keeps 🟡" divergence the doc originally claimed — no test anywhere in the suite constructs a 🟡 account to verify that specific divergence.

- **Given:** Two accounts: one 🟢 (`live-acct@test.com`), one 🔴 (`error-acct@test.com`, invalid token).
- **When-A:** `clp .usage only_valid::1`
- **When-B:** `clp .usage exclude_exhausted::1`
- **Then-A:** Exits 0; row count captured as `rows_valid`. (No explicit assertion that 🔴 is hidden from THIS output specifically — only the row count is captured for the comparison below.)
- **Then-B:** Exits 0; row count captured as `rows_excl`; 🔴 row explicitly asserted hidden (`!stdout(&out_excl).contains("error-acct@test.com")`). Test asserts `rows_excl <= rows_valid` (exclude_exhausted is at least as strict) — not an exact 🟡-divergence comparison.
- **Exit:** 0 both
- **Source fn:** `it230_lim_it_exclude_exhausted_stricter_than_only_valid` (in `usage_lim_it_test_b.rs`) — fixture is 🟢+🔴 only; verifies a `<=` row-count relationship, not the specific 🟡-divergence the doc originally claimed
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-4: `exclude_exhausted::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage exclude_exhausted::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `it175_exclude_exhausted_bad_exits_1` (in `usage_filter_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-5: `exclude_exhausted::1` with all 🔴 accounts shows 0 rows

- **Given:** Two accounts; both are 🔴.
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Table has 0 data rows (all filtered). No error.
- **Exit:** 0
- **Source fn:** `it176_exclude_exhausted_1_all_red_shows_empty` (in `usage_filter_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-6: `exclude_exhausted::true` accepted as alias for 1

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::true`
- **Then:** Exits 0. Only 🟢 row shown — same result as `exclude_exhausted::1`.
- **Exit:** 0
- **Source fn:** `it177_exclude_exhausted_true_accepted` (in `usage_filter_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-7: Cancelled account (`billing_type="none"`) hidden by `exclude_exhausted::1`

- **Given (unit test):** One `AccountQuota` with `result = Ok(OauthUsageData)` and healthy quota (`5h Left = 80%`, `7d Left = 80%`) but `account = Some(OauthAccountData { billing_type: "none", ... })` — subscription cancelled.
- **When:** `exclude_exhausted::1` filter applied (retains only accounts where `status_emoji(&aq) == "🟢"`).
- **Then:** The account is excluded — `status_emoji(&aq)` returns `"🔴"` due to the `billing_type="none"` gate (Fix BUG-317 in `format.rs`), so it fails the `== "🟢"` predicate. Without Fix(BUG-317), `status_emoji` would return `"🟢"` and the cancelled account would pass.
- **Exit:** n/a (unit test — retain predicate via status_emoji)
- **Note:** This path is transitive: `exclude_exhausted` calls `status_emoji(&aq)` which now returns 🔴 for cancelled accounts. The fix is in `format.rs`, not in the filter predicate itself.
- **Source fn:** covers via `mre_bug317_cancelled_status_emoji_is_red` (in `tests/usage/format_tests.rs`) — confirms 🔴 output; filter behavior follows from that.
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)
