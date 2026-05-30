# Test: `from_now::` Parameter

Edge case coverage for the `from_now::` parameter on `.account.renewal`. See [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `from_now::+1h30m` writes future ISO-8601 timestamp | Behavioral: write |
| EC-2 | `from_now::+0m` writes current time as `_renewal_at` | Zero Delta |
| EC-3 | `from_now::-30m` writes past timestamp (accepted verbatim) | Negative Delta |
| EC-4 | `from_now::+1d` single-unit delta accepted | Single Unit |
| EC-5 | `from_now::` combined with `at::` exits 1 | Mutual Exclusion |
| EC-6 | `from_now::` combined with `clear::` exits 1 | Mutual Exclusion |
| EC-7 | `from_now::invalid` exits 1 with parse error | Invalid Format |
| EC-8 | `from_now::+` (sign only, no units) exits 1 | Invalid Format |

---

### EC-1: `from_now::+1h30m` writes future ISO-8601 timestamp

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+1h30m`
- **Then:** Exits 0. `_renewal_at` is written as an ISO-8601 UTC string approximately 1h30m in the future (within 5s tolerance from command invocation time).
- **Exit:** 0
- **Source fn:** `ft02_account_renewal_from_now_positive` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-2: `from_now::+0m` writes current time as `_renewal_at`

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+0m`
- **Then:** Exits 0. `_renewal_at` is written as an ISO-8601 UTC timestamp within 5s of now. `.usage` would auto-advance it monthly at render time.
- **Exit:** 0
- **Source fn:** `arn24_from_now_zero_delta_writes_current_time` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-3: `from_now::-30m` writes past timestamp (accepted verbatim)

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::-30m`
- **Then:** Exits 0. `_renewal_at` is written as an ISO-8601 UTC timestamp ~30 minutes in the past. No validation error — past timestamps are accepted; auto-advance happens at read time in `.usage`.
- **Exit:** 0
- **Source fn:** `ft03_account_renewal_from_now_negative` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-4: `from_now::+1d` single-unit delta accepted

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+1d`
- **Then:** Exits 0. `_renewal_at` is written approximately 24h in the future.
- **Exit:** 0
- **Source fn:** `arn25_from_now_single_day_unit_accepted` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-5: `from_now::` combined with `at::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+1h at::2026-06-29T21:00:00Z`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft07_account_renewal_at_from_now_conflict` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-6: `from_now::` combined with `clear::` exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+1h clear::1`
- **Then:** Exits 1. Stderr names the conflicting parameters. No file written.
- **Exit:** 1
- **Source fn:** `ft09_account_renewal_from_now_clear_conflict` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-7: `from_now::invalid` exits 1 with parse error

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::invalid`
- **Then:** Exits 1. Stderr contains a parse error message. No file written.
- **Exit:** 1
- **Source fn:** `arn17_from_now_invalid_format_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### EC-8: `from_now::+` (sign only, no units) exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+`
- **Then:** Exits 1. Stderr contains a parse error message mentioning `from_now::`. No file written.
- **Note:** Previously (BUG-220), the parser returned `Ok(0)` (zero-second delta) for sign-only input, silently setting `_renewal_at` to the current time. Fixed by adding an empty-rest guard in `parse_from_now_delta`.
- **Exit:** 1
- **Source fn:** `arn26_from_now_plus_no_units_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)
