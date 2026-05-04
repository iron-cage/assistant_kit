# Test: `--effort`

Edge case coverage for the `--effort` parameter. See [params.md](../../../../docs/cli/params.md#parameter--17---effort) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default invocation → `--effort max` present in assembled command | Default Behavior |
| EC-2 | `--effort medium` → `--effort medium` in command (not `--effort max`) | Override |
| EC-3 | `--effort high` → `--effort high` in command | Override |
| EC-4 | `--effort low` → `--effort low` in command | Override |
| EC-5 | `--effort max` explicit → `--effort max` (idempotent with default) | Idempotent |
| EC-6 | `--effort invalid` → exit 1, stderr lists valid values | Validation |
| EC-7 | `--effort` with no value → exit 1, "requires a value" error | Validation |
| EC-8 | `--help` output contains `--effort` | Documentation |

## Test Coverage Summary

- Default Behavior: 1 test
- Override: 3 tests
- Idempotent: 1 test
- Validation: 2 tests
- Documentation: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Default invocation → `--effort max` present

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug"`
- **Then:** Assembled command contains `--effort max`.; `--effort max` present in assembled command
- **Exit:** 0
- **Source:** [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-2: `--effort medium` overrides default

- **Given:** clean environment
- **When:** `clr --dry-run --effort medium "Fix the bug"`
- **Then:** `--effort medium` present; `--effort max` absent.; override applied correctly
- **Exit:** 0
- **Source:** [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort)

---

### EC-3: `--effort high` override

- **Given:** clean environment
- **When:** `clr --dry-run --effort high "Fix the bug"`
- **Then:** `--effort high` present.; high level accepted and used
- **Exit:** 0
- **Source:** [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort)

---

### EC-4: `--effort low` override

- **Given:** clean environment
- **When:** `clr --dry-run --effort low "Fix the bug"`
- **Then:** `--effort low` present.; low level accepted and used
- **Exit:** 0
- **Source:** [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort)

---

### EC-5: `--effort max` explicit is idempotent

- **Given:** clean environment
- **When:** `clr --dry-run --effort max "Fix the bug"`
- **Then:** `--effort max` appears exactly once.; exactly one `--effort max` in output
- **Exit:** 0
- **Source:** [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort)

---

### EC-6: `--effort invalid` → validation error

- **Given:** clean environment
- **When:** `clr --effort bad_level "Fix the bug"`
- **Then:** Exit 1; stderr contains "valid values" and/or the list `low, medium, high, max`.; error message references valid levels
- **Exit:** 1
- **Source:** [types.md — EffortLevel validation errors](../../../../docs/cli/types.md#type--7-effortlevel)

---

### EC-7: `--effort` with no value → missing value error

- **Given:** clean environment
- **When:** `clr --effort`
- **Then:** Exit 1; stderr contains "requires a value".; error message is clear about missing value
- **Exit:** 1
- **Source:** [params.md — --effort (Validation)](../../../../docs/cli/params.md#parameter--17---effort)
**Automated Test:** `effort_args_test.rs::t67_effort_missing_value_rejected`

---

### EC-8: `--help` lists `--effort`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--effort`.; flag present in help
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)
