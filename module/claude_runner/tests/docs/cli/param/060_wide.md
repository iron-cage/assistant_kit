# Parameter :: `--wide`

Edge case coverage for the `--wide` parameter. See [060_wide.md](../../../../docs/cli/param/060_wide.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr ps --wide` shows all 11 columns including Mode, Command, Binary | Behavioral |
| EC-2 | `clr ps -w` short form shows all 11 columns | Behavioral |
| EC-3 | `clr ps --wide` with `--columns pid,task` → `--columns` wins | Precedence |
| EC-4 | `clr ps` without `--wide` hides Mode, Command, Binary columns | Default |
| EC-5 | `clr ps --help` output contains `--wide` / `-w` | Documentation |

## Test Coverage Summary

- Behavioral: 2 tests (EC-1, EC-2)
- Precedence: 1 test (EC-3)
- Default: 1 test (EC-4)
- Documentation: 1 test (EC-5)

**Total:** 5 edge cases

---

### EC-1: `--wide` shows all 11 columns

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --wide`
- **Expected behavior:** Exit 0; stdout contains `PID`, `Elapsed`, `CPU%`, `RAM`, `State`, `Absolute Path`, `Task`, `Mode`, `Command`, `Binary` headers
- **Exit:** 0
- **Source:** [060_wide.md](../../../../docs/cli/param/060_wide.md)

---

### EC-2: `-w` short form

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps -w`
- **Expected behavior:** Exit 0; stdout contains `Mode`, `Command`, `Binary` headers (same as `--wide`)
- **Exit:** 0
- **Source:** [060_wide.md](../../../../docs/cli/param/060_wide.md)

---

### EC-3: `--columns` overrides `--wide`

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --wide --columns pid,task`
- **Expected behavior:** Exit 0; stdout contains `PID` and `Task`; stdout does NOT contain `Mode`, `Command`, `Binary`
- **Exit:** 0
- **Source:** [060_wide.md](../../../../docs/cli/param/060_wide.md)

---

### EC-4: Default hides optional columns

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps` (no `--wide`)
- **Expected behavior:** Exit 0; stdout does NOT contain `Mode`, `Command`, `Binary`
- **Exit:** 0
- **Source:** [060_wide.md](../../../../docs/cli/param/060_wide.md)

---

### EC-5: Help output contains `--wide`

- **Command:** `clr ps --help`
- **Expected behavior:** Exit 0; stdout contains `--wide` and `-w`
- **Exit:** 0
- **Source:** [060_wide.md](../../../../docs/cli/param/060_wide.md)
