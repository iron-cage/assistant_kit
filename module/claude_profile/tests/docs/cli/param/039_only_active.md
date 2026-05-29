# Test: `only_active::` Parameter

Edge case coverage for the `only_active::` parameter on `.usage`. See [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `only_active::1` shows exactly the active account row | Boolean Filter |
| EC-2 | `only_active::0` (default) shows all rows | Default No-op |
| EC-3 | `only_active::bad` exits 1 naming valid values | Invalid Value |

---

### EC-1: `only_active::1` shows exactly the active account row

- **Given:** Three accounts; one is the active account.
- **When:** `clp .usage only_active::1`
- **Then:** Exits 0. Exactly one row in the table body — the active account.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-2: `only_active::0` shows all rows

- **Given:** Three accounts.
- **When:** `clp .usage only_active::0`
- **Then:** Exits 0. All rows shown (default behavior).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-3: `only_active::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage only_active::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)
