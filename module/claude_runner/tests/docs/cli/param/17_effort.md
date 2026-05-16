# Parameter :: `--effort`

Edge case coverage for the `--effort` parameter. See [17_effort.md](../../../../docs/cli/param/17_effort.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default invocation → `--effort max` present in assembled command | Behavioral Divergence |
| EC-2 | `--effort medium` → `--effort medium` in command (not `--effort max`) | Behavioral Divergence |
| EC-3 | `--effort high` → `--effort high` in command | Override |
| EC-4 | `--effort low` → `--effort low` in command | Override |
| EC-5 | `--effort max` explicit → `--effort max` (idempotent with default) | Idempotent |
| EC-6 | `--effort invalid` → exit 1, stderr lists valid values | Validation |
| EC-7 | `--effort` with no value → exit 1, "requires a value" error | Validation |
| EC-8 | `--help` output contains `--effort` | Documentation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Override: 2 tests (EC-3, EC-4)
- Idempotent: 1 test
- Validation: 2 tests
- Documentation: 1 test

**Total:** 8 edge cases


---

### EC-1: Default invocation → `--effort max` present

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug"`
- **Then:** Assembled command contains `--effort max`.; `--effort max` present in assembled command
- **Exit:** 0
- **Source:** [--effort](../../../../docs/cli/param/17_effort.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-2: `--effort medium` overrides default

- **Given:** clean environment
- **When:** `clr --dry-run --effort medium "Fix the bug"`
- **Then:** `--effort medium` present; `--effort max` absent.; override applied correctly
- **Exit:** 0
- **Source:** [--effort](../../../../docs/cli/param/17_effort.md)

---

### EC-3: `--effort high` override

- **Given:** clean environment
- **When:** `clr --dry-run --effort high "Fix the bug"`
- **Then:** `--effort high` present.; high level accepted and used
- **Exit:** 0
- **Source:** [--effort](../../../../docs/cli/param/17_effort.md)

---

### EC-4: `--effort low` override

- **Given:** clean environment
- **When:** `clr --dry-run --effort low "Fix the bug"`
- **Then:** `--effort low` present.; low level accepted and used
- **Exit:** 0
- **Source:** [--effort](../../../../docs/cli/param/17_effort.md)

---

### EC-5: `--effort max` explicit is idempotent

- **Given:** clean environment
- **When:** `clr --dry-run --effort max "Fix the bug"`
- **Then:** `--effort max` appears exactly once.; exactly one `--effort max` in output
- **Exit:** 0
- **Source:** [--effort](../../../../docs/cli/param/17_effort.md)

---

### EC-6: `--effort invalid` → validation error

- **Given:** clean environment
- **When:** `clr --effort bad_level "Fix the bug"`
- **Then:** Exit 1; stderr contains "valid values" and/or the list `low, medium, high, max`.; error message references valid levels
- **Exit:** 1
- **Source:** [type.md — EffortLevel validation errors](../../../../docs/cli/type.md#type--7-effortlevel)

---

### EC-7: `--effort` with no value → missing value error

- **Given:** clean environment
- **When:** `clr --effort`
- **Then:** Exit 1; stderr contains "requires a value".; error message is clear about missing value
- **Exit:** 1
- **Source:** [--effort (Validation)](../../../../docs/cli/param/17_effort.md)

---

### EC-8: `--help` lists `--effort`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--effort`.; flag present in help
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)
