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
- **Source fn:** `it231_lim_it_get_7d_left_extracts_bare_pct` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-2: `get::account` extracts bare account name

- **Given:** Two accounts `alice@test.com` and `bob@test.com`; `sort::name` so alice is first.
- **When:** `clp .usage sort::name get::account`
- **Then:** Exits 0. Stdout is exactly `alice@test.com` (bare string, no other output).
- **Exit:** 0
- **Source fn:** `it190_get_account_extracts_first_name` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-3: `get::status` extracts bare status emoji

- **Given:** One 🟢 account.
- **When:** `clp .usage get::status`
- **Then:** Exits 0. Stdout contains `🟢` (single emoji).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it232_lim_it_get_status_extracts_green_emoji` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-4: `get::` with empty filtered result outputs nothing

- **Given:** One account with `5h Left = 10%`.
- **When:** `clp .usage min_5h::50 get::7d_left`
- **Then:** Exits 0. Stdout is empty (no rows passed filter, nothing to extract).
- **Exit:** 0
- **Source fn:** `it193_get_with_empty_filtered_result_empty_stdout` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-5: `get::bogus` exits 1 listing valid field IDs

- **Given:** Any environment.
- **When:** `clp .usage get::bogus`
- **Then:** Exits 1. Stderr lists valid field IDs: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `sub`, `status`, `account`, `host`, `role`, `next_event_type`, `next_event_secs`.
- **Exit:** 1
- **Source fn:** `it233_get_bogus_exits_1_names_valid_fields` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-6: `get::` output contains no table chrome

- **Given:** Two accounts.
- **When:** `clp .usage get::account`
- **Then:** Exits 0. Stdout does NOT contain column header names ("5h Left", "7d Left", etc.), separator lines (`---`), or footer text ("Valid:", "Next by strategy:").
- **Exit:** 0
- **Source fn:** `it191_get_account_no_table_chrome` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)

---

### EC-7: `get::next_event_type` and `get::next_event_secs` extract next-event scalars

- **Given:** One account with a known upcoming 7d quota reset (`seven_day.resets_at` set to a future timestamp ~2 days away); `_renewal_at` not set.
- **When (a):** `clp .usage get::next_event_type`
- **Then (a):** Exits 0. Stdout is `+7d` (the event type label for the soonest upcoming strategic event).
- **When (b):** `clp .usage get::next_event_secs`
- **Then (b):** Exits 0. Stdout is a bare integer (seconds to event, e.g. `172800`); no table chrome.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it234_lim_it_get_next_event_type_and_secs` (in `tests/cli/usage_test.rs`)
- **Source:** [param/045_get.md](../../../../docs/cli/param/045_get.md)
