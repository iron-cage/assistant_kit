# Test: Invariant — Default Flags

Test case planning for [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md). Tests validate that each default flag injection invariant holds and that opt-out flags correctly suppress each default.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `-c` present by default (continuation invariant) | Default Present |
| IT-2 | `--dangerously-skip-permissions` present by default | Default Present |
| IT-3 | `--chrome` present by default | Default Present |
| IT-4 | `--effort max` present by default | Default Present |
| IT-5 | Message has `ultrathink` suffix by default | Default Present |
| IT-6 | All opt-outs together remove all suppressible defaults | Combined Suppression |

## Test Coverage Summary

- Default Present: 5 tests (IT-1, IT-2, IT-3, IT-4, IT-5)
- Combined Suppression: 1 test (IT-6)

**Total:** 6 tests


---

### IT-1: `-c` present by default (continuation invariant)

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains ` -c`; continuation flag injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-2: `--dangerously-skip-permissions` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--dangerously-skip-permissions`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-3: `--chrome` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--chrome`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-4: `--effort max` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--effort max`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-5: Message has `ultrathink` suffix by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the auth bug"`
- **Then:** Message argument in assembled command ends with `ultrathink` suffix (appended as `\n\nultrathink`); suffix injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-6: All opt-outs together remove all suppressible defaults

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --no-skip-permissions --no-ultrathink --no-effort-max "Fix bug"`
- **Then:** Assembled command does NOT contain `-c`, does NOT contain `--dangerously-skip-permissions`, does NOT contain `--effort`, and message does NOT have `ultrathink` suffix; all suppressible defaults removed
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)
