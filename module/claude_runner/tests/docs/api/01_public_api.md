# Test: API — Public API

Test case planning for [api/001_public_api.md](../../../../docs/api/001_public_api.md). Tests validate the three public API surface contracts: `COMMANDS_YAML` constant, `VerbosityLevel` newtype, and `register_commands` function.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AP-1 | `COMMANDS_YAML` resolves to a readable YAML string | COMMANDS_YAML |
| AP-2 | YAML at `COMMANDS_YAML` is well-formed (parseable) | COMMANDS_YAML |
| AP-3 | `VerbosityLevel::default()` equals level 3 | VerbosityLevel |
| AP-4 | `--verbosity 0` accepted (lower boundary) | VerbosityLevel |
| AP-5 | `--verbosity 6` rejected → exit 1 (above maximum) | VerbosityLevel |
| AP-6 | `register_commands` callable; returns without error | register_commands |
| AP-7 | `--verbosity 5` accepted (upper boundary) | VerbosityLevel |

## Test Coverage Summary

- COMMANDS_YAML: 2 tests (AP-1, AP-2)
- VerbosityLevel: 4 tests (AP-3, AP-4, AP-5, AP-7)
- register_commands: 1 test (AP-6)

**Total:** 7 tests


---

### AP-1: `COMMANDS_YAML` resolves to a readable YAML string

- **Given:** clean environment
- **When:** access `claude_runner::COMMANDS_YAML` constant at compile or runtime
- **Then:** The constant is a non-empty `&str`; the string is accessible and non-null
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-2: YAML at `COMMANDS_YAML` is well-formed

- **Given:** clean environment
- **When:** parse `claude_runner::COMMANDS_YAML` with a YAML parser
- **Then:** Parse succeeds with no errors; the YAML structure is valid
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-3: `VerbosityLevel::default()` equals level 3

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"` (no `--verbosity` flag)
- **Then:** Behavior matches verbosity level 3 (the default); `VerbosityLevel::default()` produces the same level as an explicit `--verbosity 3`
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-4: `--verbosity 0` accepted (lower boundary)

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 0 "Fix bug"`
- **Then:** Exit 0; lower boundary value 0 is accepted without error
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-5: `--verbosity 6` rejected → exit 1

- **Given:** clean environment
- **When:** `clr --verbosity 6 "Fix bug"`
- **Then:** Exit code 1; stderr contains error indicating verbosity value is out of range (maximum is 5)
- **Exit:** 1
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-6: `register_commands` callable without error

- **Given:** clean environment
- **When:** call `claude_runner::register_commands()` in a unit test
- **Then:** Function returns without panic or error; return type is unit (`()`); no side effects that cause failure
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)

---

### AP-7: `--verbosity 5` accepted (upper boundary)

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 5 "Fix bug"`
- **Then:** Exit 0; upper boundary value 5 is accepted without error
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../../docs/api/001_public_api.md)
