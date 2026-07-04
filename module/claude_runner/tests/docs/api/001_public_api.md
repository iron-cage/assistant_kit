# Test: API — Public API

Test case planning for [api/001_public_api.md](../../../docs/api/001_public_api.md). Tests validate the two public API surface contracts: `COMMANDS_YAML` constant and `register_commands` function.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AP-1 | `COMMANDS_YAML` resolves to a readable YAML string | COMMANDS_YAML |
| AP-2 | YAML at `COMMANDS_YAML` is well-formed (parseable) | COMMANDS_YAML |
| AP-6 | `register_commands` callable; returns without error | register_commands |

## Test Coverage Summary

- COMMANDS_YAML: 2 tests (AP-1, AP-2)
- register_commands: 1 test (AP-6)

**Total:** 3 tests

**Note:** AP-3, AP-4, AP-5, AP-7 previously tested `VerbosityLevel` (0–5 boundary validation). Removed when `VerbosityLevel` was deleted from the public API (TSK-337). CLI `--quiet` behavior is covered by `quiet_test.rs` (QT-1–QT-6).


---

### AP-1: `COMMANDS_YAML` resolves to a readable YAML string

- **Given:** clean environment
- **When:** access `claude_runner::COMMANDS_YAML` constant at compile or runtime
- **Then:** The constant is a non-empty `&str`; the string is accessible and non-null
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../docs/api/001_public_api.md)

---

### AP-2: YAML at `COMMANDS_YAML` is well-formed

- **Given:** clean environment
- **When:** parse `claude_runner::COMMANDS_YAML` with a YAML parser
- **Then:** Parse succeeds with no errors; the YAML structure is valid
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../docs/api/001_public_api.md)

---

### AP-6: `register_commands` callable without error

- **Given:** clean environment
- **When:** call `claude_runner::register_commands()` in a unit test
- **Then:** Function returns without panic or error; return type is unit (`()`); no side effects that cause failure
- **Exit:** 0
- **Source:** [api/001_public_api.md](../../../docs/api/001_public_api.md)


