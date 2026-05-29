# Test: `only_next::` Parameter

Edge case coverage for the `only_next::` parameter on `.usage`. See [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `only_next::1` shows exactly the → row | Boolean Filter |
| EC-2 | `only_next::1` with no eligible candidate shows 0 rows, exits 0 | Empty Result |
| EC-3 | `only_next::1 next::drain` shows → row from drain strategy | Strategy Composition |
| EC-4 | `only_next::bad` exits 1 naming valid values | Invalid Value |

---

### EC-1: `only_next::1` shows exactly the → row

- **Given:** Two accounts with valid quota; one receives →.
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Exactly one row shown — the → account. Footer still shown.
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-2: `only_next::1` with no eligible candidate shows 0 rows

- **Given:** One account that is `is_current=true` (no eligible candidate for `next::` strategy).
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Table has 0 data rows. No error.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-3: `only_next::1 next::drain` shows → row from drain strategy

- **Given:** Two accounts with valid quota; drain strategy winner differs from default renew winner.
- **When:** `clp .usage only_next::1 next::drain`
- **Then:** Exits 0. The row shown is the drain strategy winner (not the renew default winner).
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)

---

### EC-4: `only_next::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage only_next::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/040_only_next.md](../../../../docs/cli/param/040_only_next.md)
