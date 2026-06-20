# Test: `only_next::` Parameter

Edge case coverage for the `only_next::` parameter on `.usage`. See [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `only_next::1` shows exactly the recommended account row | Behavioral Divergence |
| EC-2 | `only_next::1` with no eligible candidate shows 0 rows, exits 0 | Empty Result |
| EC-3 | `only_next::1 sort::renews` shows recommended account row from renews strategy | Strategy Composition |
| EC-4 | `only_next::bad` exits 1 naming valid values | Invalid Value |
| EC-5 | `only_next::0` (default) shows all rows | Behavioral Divergence |
| EC-6 | `only_next::true` accepted (alias for 1) | Alias Acceptance |

---

### EC-1: `only_next::1` shows exactly the recommended account row

- **Given:** Two accounts with valid quota; one is the footer recommendation.
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Exactly one row shown — the footer-recommended account. Footer still shown.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it206_lim_it_ft028_04_only_next_1_shows_arrow` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-2: `only_next::1` with no eligible candidate shows 0 rows

- **Given:** One account that is `is_current=true` (no eligible candidate for the active sort strategy).
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Table has 0 data rows. No error.
- **Exit:** 0
- **Source fn:** `it160_only_next_1_no_valid_accounts_shows_empty` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-3: `only_next::1 sort::renews` shows recommended account row from renews strategy

- **Given:** Two accounts with valid quota; renews strategy selects by billing renewal date.
- **When:** `clp .usage only_next::1 sort::renews`
- **Then:** Exits 0. Exactly one row shown — the renews-strategy winner — the footer-recommended account.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it226_lim_it_only_next_1_renews_shows_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-4: `only_next::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage only_next::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `it161_only_next_bad_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-5: `only_next::0` shows all rows (default behavior)

- **Given:** Two accounts with valid quota.
- **When:** `clp .usage only_next::0`
- **Then:** Exits 0. Both rows shown — the recommended account and the non-recommended account.
- **Exit:** 0
- **Source fn:** `it162_only_next_0_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-6: `only_next::true` accepted as alias for 1

- **Given:** Two accounts with valid quota; one is the footer recommendation.
- **When:** `clp .usage only_next::true`
- **Then:** Exits 0. Exactly one row shown — same result as `only_next::1`.
- **Exit:** 0
- **Source fn:** `it227_lim_it_only_next_true_shows_arrow_row` (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)
