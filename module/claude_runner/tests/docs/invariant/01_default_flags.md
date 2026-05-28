# Test: Invariant — Default Flags

Test case planning for [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md). Tests validate that each default flag injection invariant holds and that opt-out flags correctly suppress each default.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `-c` present by default (continuation invariant) | Default Present |
| IN-2 | `--dangerously-skip-permissions` present by default | Default Present |
| IN-3 | `--chrome` present by default | Default Present |
| IN-4 | `--effort max` present by default | Default Present |
| IN-5 | Message has `ultrathink` suffix by default | Default Present |
| IN-6 | All opt-outs together remove all suppressible defaults | Combined Suppression |
| IN-7 | Empty `--session-dir` → `-c` absent from assembled command (BUG-214 regression) | First-use guard |

## Test Coverage Summary

- Default Present: 5 tests (IN-1, IN-2, IN-3, IN-4, IN-5)
- Combined Suppression: 1 test (IN-6)
- First-use guard: 1 test (IN-7)

**Total:** 7 tests


---

### IN-1: `-c` present by default (continuation invariant)

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains ` -c`; continuation flag injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-2: `--dangerously-skip-permissions` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--dangerously-skip-permissions`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-3: `--chrome` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--chrome`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-4: `--effort max` present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--effort max`; injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-5: Message has `ultrathink` suffix by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the auth bug"`
- **Then:** Message argument in assembled command ends with `ultrathink` suffix (appended as `\n\nultrathink`); suffix injected by default
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-6: All opt-outs together remove all suppressible defaults

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --no-skip-permissions --no-ultrathink --no-effort-max --no-chrome "Fix bug"`
- **Then:** Assembled command does NOT contain `-c`, does NOT contain `--dangerously-skip-permissions`, does NOT contain `--chrome`, does NOT contain `--effort`, and message does NOT have `ultrathink` suffix; all suppressible defaults removed
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IN-7: Empty `--session-dir` → `-c` absent from assembled command (BUG-214 regression)

- **Given:** clean environment; `--session-dir` points to a freshly created empty directory
- **When:** `clr --dry-run --session-dir /tmp/mre214_empty "Fix bug"`
- **Then:** Assembled command does NOT contain ` -c`; `session_exists()` guard detected empty directory and suppressed `-c` injection
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md § Fixed Defects](../../../../docs/invariant/001_default_flags.md)
