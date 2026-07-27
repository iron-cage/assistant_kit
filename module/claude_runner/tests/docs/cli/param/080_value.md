# Parameter :: `--value`

Edge case coverage for the `--value` parameter. See [080_value.md](../../../../docs/cli/param/080_value.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr tools --value name` prints bare names, one per line, no table decoration | Behavioral |
| EC-2 | `clr tools --name Bash --value category` — filter narrows to one row, single bare value printed | Interaction |
| EC-3 | `clr tools --value badkey` exits 1 with error listing valid keys | Validation |
| EC-4 | `clr tools --value name --inspect` exits 1 (mutually exclusive) | Validation |
| EC-5 | `clr tools --columns name --value category` — `--columns` ignored when `--value` active | Precedence |
| EC-6 | `clr tools --name doesnotexist --value name` — zero matches, no output, exit 0 | Boundary |
| EC-7 | `clr tools --help` output contains `--value` | Documentation |

## Test Coverage Summary

- Behavioral: 1 test (EC-1)
- Interaction: 1 test (EC-2)
- Validation: 2 tests (EC-3, EC-4)
- Precedence: 1 test (EC-5)
- Boundary: 1 test (EC-6)
- Documentation: 1 test (EC-7)

**Total:** 7 edge cases

---

### EC-1: Bare value output, one per line

- **Command:** `clr tools --value name`
- **Expected behavior:** Exit 0; stdout contains `Bash` on its own line; stdout does NOT contain table header text (`#`, `Category`, `Description`) or the table caption
- **Exit:** 0
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-2: Single-row filter narrows to exactly one bare value

- **Command:** `clr tools --name Bash --value category`
- **Expected behavior:** Exit 0; stdout is exactly `Shell` (plus trailing newline)
- **Exit:** 0
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-3: Unknown `--value` key rejected

- **Command:** `clr tools --value badkey`
- **Expected behavior:** Exit 1; stderr lists valid keys (`idx`, `name`, `category`, `desc`)
- **Exit:** 1
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-4: `--value` and `--inspect` are mutually exclusive

- **Command:** `clr tools --value name --inspect`
- **Expected behavior:** Exit 1; stderr states the two flags cannot be combined
- **Exit:** 1
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-5: `--columns` is ignored when `--value` is active

- **Command:** `clr tools --columns name --value category`
- **Expected behavior:** Exit 0; stdout contains bare `category` values (e.g. `Shell`), not values restricted to a `name`-column table
- **Exit:** 0
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-6: Zero matches produces no output

- **Command:** `clr tools --name doesnotexist --value name`
- **Expected behavior:** Exit 0; stdout is empty
- **Exit:** 0
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)

---

### EC-7: Help output contains `--value`

- **Command:** `clr tools --help`
- **Expected behavior:** Exit 0; stdout contains `--value`
- **Exit:** 0
- **Source:** [080_value.md](../../../../docs/cli/param/080_value.md)
