# Test: `interval::`

Edge case coverage for the `interval::` parameter. See [param/readme.md](../../../../docs/cli/param/08_interval.md) and [type/readme.md](../../../../docs/cli/type/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `interval::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `interval::`.
- **Commands:** `.version.guard`
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `interval::0` behaves as one-shot (default) | Default Behavior |
| EC-2 | Default (omitted) resolves to `interval::0` | Default Behavior |
| EC-3 | `interval::` (empty) rejected with exit 1 and message | Empty Value |
| EC-4 | `interval::-1` rejected — negative values not accepted | Invalid Value |
| EC-5 | `interval::abc` rejected — non-integer not accepted | Format Violation |
| EC-6 | `interval::5` starts watch mode (process does not exit immediately) | Watch Mode |
| EC-7 | `interval::0` with `dry::1` combines correctly | Interaction |
| EC-8 | `interval::` only accepted by `.version.guard` | Command Scope |
| EC-9 | `interval::18446744073709551615` (u64::MAX) → exit 1 | Overflow: above i64::MAX |

## Test Coverage Summary

- Default Behavior: 2 tests
- Empty Value: 1 test
- Invalid Value: 1 test
- Format Violation: 1 test
- Watch Mode: 1 test
- Interaction: 1 test
- Command Scope: 1 test
- Overflow (above i64::MAX): 1 test

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-1 (`interval::0` → one-shot, process exits immediately) ↔ EC-6 (`interval::5` → watch mode, process stays alive until killed)

---

### EC-1: `interval::0` behaves as one-shot (default)

- **Given:** No preferred version set.
- **When:** `clv .version.guard interval::0`
- **Then:** output contains "stable" (defaults to stable); process exits immediately.; one-shot behavior; immediate exit
- **Exit:** 0
- **Source:** [param/readme.md — interval:: default: 0](../../../../docs/cli/param/08_interval.md)

---

### EC-2: Default (omitted) resolves to `interval::0`

- **Given:** No preferred version set.
- **When:** `clv .version.guard`
- **Then:** output contains "stable" (defaults to stable); process exits immediately.; one-shot mode; same as explicit `interval::0`
- **Exit:** 0
- **Source:** [param/readme.md — interval:: default](../../../../docs/cli/param/08_interval.md)

---

### EC-3: `interval::` (empty) rejected with exit 1 and message

- **Given:** clean environment
- **When:** `clv .version.guard interval::`
- **Then:** stderr: error about `interval::` requiring a value; exit code 1.; usage error
- **Exit:** 1
- **Source:** [param/readme.md — interval:: constraints](../../../../docs/cli/param/08_interval.md)

---

### EC-4: `interval::-1` rejected — negative values not accepted

- **Given:** clean environment
- **When:** `clv .version.guard interval::-1`
- **Then:** stderr: error about invalid `interval::` value; exit code 1.; negative value rejected
- **Exit:** 1
- **Source:** [param/readme.md — interval:: type: u64](../../../../docs/cli/param/08_interval.md)

---

### EC-5: `interval::abc` rejected — non-integer not accepted

- **Given:** clean environment
- **When:** `clv .version.guard interval::abc`
- **Then:** stderr: error about invalid `interval::` format; exit code 1.; string value rejected
- **Exit:** 1
- **Source:** [param/readme.md — interval:: type: u64](../../../../docs/cli/param/08_interval.md)

---

### EC-6: `interval::5` starts watch mode (process does not exit immediately)

- **Given:** No preferred version set.
- **When:** `timeout 3 clv .version.guard interval::5`
- **Then:** Process produces at least one status line; continues running until killed by `timeout`.; Process stays alive; watch mode active
- **Exit:** 0
- **Source:** [param/readme.md — interval:: watch mode](../../../../docs/cli/param/08_interval.md)

---

### EC-7: `interval::0` with `dry::1` combines correctly

- **Given:** Preferred version set in settings.
- **When:** `clv .version.guard interval::0 dry::1`
- **Then:** `[dry-run]` prefixed output; process exits immediately.; dry-run markers present; one-shot exit
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+interval](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-8: `interval::` only accepted by `.version.guard`

- **Given:** clean environment
- **When:** `clv .version.install interval::5`
- **Then:** stderr: error about unknown parameter `interval::`; exit code 1.; parameter rejected by non-guard command
- **Exit:** 1
- **Source:** [command/readme.md — .version.install parameters](../../../../docs/cli/command/version.md#command-4-versioninstall)

---

### EC-9: `interval::18446744073709551615` (u64::MAX) → exit 1

- **Given:** clean environment
- **When:** `clv .version.guard interval::18446744073709551615`
- **Then:** exit code 1; error message mentions `interval::`, NOT "fit in target type".; error is user-friendly (no internal unilang message)
- **Exit:** 1
- **Source:** [param/09_count.md — EC-6 sibling pattern](../../../../docs/cli/param/09_count.md)
- **Note:** Gap Class — implemented regression test (`tc491_interval_u64_max_rejected_with_clear_error`) had no corresponding documented case in this spec's Test Case Index, unlike its sibling `count::` parameter (09_count.md EC-6). Source: BUG-003.

---

### Source Functions

| Function | File |
|----------|------|
| `tc409_guard_interval_zero_oneshot` | `tests/cli/mutation_version_guard_test.rs` |
| `tc415_watch_loop_continues_after_install_error` | `tests/cli/mutation_version_guard_test.rs` |
| `tc491_interval_u64_max_rejected_with_clear_error` | `cli_args_test/param_numeric_test.rs` |
