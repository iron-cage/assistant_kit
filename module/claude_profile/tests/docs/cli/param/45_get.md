# Test: `get::` Parameter

Edge case coverage for the `get::` parameter on `.usage`. See [param/045_get.md](../../../../docs/cli/param/045_get.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `get::7d_left` extracts bare percentage, no headers | Behavioral Divergence |
| EC-2 | `get::account` extracts bare account name | Extraction |
| EC-3 | `get::status` extracts bare status emoji | Extraction |
| EC-4 | `get::` with empty filtered result outputs nothing, exits 0 | Empty Result |
| EC-5 | `get::bogus` exits 1 listing valid field IDs | Invalid Value |
| EC-6 | `get::` output contains no table chrome | Behavioral Divergence |
| EC-7 | `get::next_event_type` and `get::next_event_secs` extract next-event scalars | New Field IDs |

---

### EC-1: `get::7d_left` extracts bare percentage string

- **Given:** One account with live quota; `sort::name` for determinism.
- **When:** `clp .usage sort::name get::7d_left`
- **Then:** Exits 0. Stdout is a single percentage string (e.g., `65%`). No column headers, no separator line, no footer.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it231_lim_it_get_7d_left_extracts_bare_pct` (in `usage_lim_it_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-2: `get::account` extracts bare account name

- **Given:** Two accounts `alice@test.com` and `bob@test.com`; `sort::name` so alice is first.
- **When:** `clp .usage sort::name get::account`
- **Then:** Exits 0. Stdout is exactly `alice@test.com` (bare string, no other output).
- **Exit:** 0
- **Source fn:** `it190_get_account_extracts_first_name` (in `usage_filter_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-3: `get::status` extracts bare status emoji

- **Given:** One 🟢 account.
- **When:** `clp .usage get::status`
- **Then:** Exits 0. Stdout contains `🟢` (single emoji).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it232_lim_it_get_status_extracts_green_emoji` (in `usage_lim_it_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-4: `get::` with empty filtered result outputs nothing

- **Given:** One account with `5h Left = 10%`.
- **When:** `clp .usage min_5h::50 get::7d_left`
- **Then:** Exits 0. Stdout is empty (no rows passed filter, nothing to extract).
- **Exit:** 0
- **Source fn:** `it193_get_with_empty_filtered_result_empty_stdout` (in `usage_filter_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-5: `get::bogus` exits 1 listing valid field IDs

- **Given:** Any environment.
- **When:** `clp .usage get::bogus`
- **Then:** Exits 1. Stderr lists valid field IDs: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `sub`, `status`, `account`, `host`, `role`, `next_event_type`, `next_event_secs`.
- **Exit:** 1
- **Source fn:** `it233_get_bogus_exits_1_names_valid_fields` (in `usage_lim_it_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-6: `get::` output contains no table chrome

- **Given:** Two accounts.
- **When:** `clp .usage get::account`
- **Then:** Exits 0. Stdout does NOT contain column header names ("5h Left", "7d Left", etc.), separator lines (`---`), or footer text ("Valid:", "Next by strategy:").
- **Exit:** 0
- **Source fn:** `it191_get_account_no_table_chrome` (in `usage_filter_test_b.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-7: `get::next_event_type` and `get::next_event_secs` extract next-event scalars (live account — uncontrolled precondition)

> **Semantic drift correction:** the cited test does not construct an account with a controlled, known upcoming 7d reset. It uses a live account (shared token) whose `seven_day.resets_at` is whatever the real API returns — not a fixed ~2-day-away timestamp. Consequently the test tolerates EITHER `"+7d"` OR `"$ren"` as valid `next_event_type` output (`let valid_labels = ["+7d", "$ren"]`), not the single deterministic `"+7d"` value originally claimed. `next_event_secs` is verified only as "parses as *some* valid `u64`" (`secs_str.parse::<u64>().is_ok()`), not the specific example value (`172800`) originally implied.

- **Given:** One live account (shared token) — `seven_day.resets_at` and any `_renewal_at` state are whatever the live API returns; not controlled by the test.
- **When (a):** `clp .usage get::next_event_type`
- **Then (a):** Exits 0. Stdout is EITHER `+7d` OR `$ren` (the test accepts either label as valid — it does not force a specific one).
- **When (b):** `clp .usage get::next_event_secs`
- **Then (b):** Exits 0. Stdout parses as some valid non-negative integer (`u64`); no table chrome. The specific value is not asserted.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it234_lim_it_get_next_event_type_and_secs` (in `usage_lim_it_test_b.rs`) — uses an uncontrolled live account; tolerates either of two output labels rather than asserting the single deterministic value originally claimed
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)
