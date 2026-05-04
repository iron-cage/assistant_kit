# Test: `interval::`

Edge case coverage for the `interval::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--9-interval) and [types.md](../../../../docs/cli/types.md) for specification.

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

## Test Coverage Summary

- Default Behavior: 2 tests
- Empty Value: 1 test
- Invalid Value: 1 test
- Format Violation: 1 test
- Watch Mode: 1 test
- Interaction: 1 test
- Command Scope: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `interval::0` behaves as one-shot (default)

- **Given:** No preferred version set.
- **When:** `cm .version.guard interval::0`
- **Then:** output contains "stable" (defaults to stable); process exits immediately.; one-shot behavior; immediate exit
- **Exit:** 0
- **Source:** [params.md — interval:: default: 0](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-2: Default (omitted) resolves to `interval::0`

- **Given:** No preferred version set.
- **When:** `cm .version.guard`
- **Then:** output contains "stable" (defaults to stable); process exits immediately.; one-shot mode; same as explicit `interval::0`
- **Exit:** 0
- **Source:** [params.md — interval:: default](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-3: `interval::` (empty) rejected with exit 1 and message

- **Given:** clean environment
- **When:** `cm .version.guard interval::`
- **Then:** stderr: error about `interval::` requiring a value; exit code 1.; usage error
- **Exit:** 1
- **Source:** [params.md — interval:: constraints](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-4: `interval::-1` rejected — negative values not accepted

- **Given:** clean environment
- **When:** `cm .version.guard interval::-1`
- **Then:** stderr: error about invalid `interval::` value; exit code 1.; negative value rejected
- **Exit:** 1
- **Source:** [params.md — interval:: type: u64](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-5: `interval::abc` rejected — non-integer not accepted

- **Given:** clean environment
- **When:** `cm .version.guard interval::abc`
- **Then:** stderr: error about invalid `interval::` format; exit code 1.; string value rejected
- **Exit:** 1
- **Source:** [params.md — interval:: type: u64](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-6: `interval::5` starts watch mode (process does not exit immediately)

- **Given:** No preferred version set.
- **When:** `timeout 3 cm .version.guard interval::5`
- **Then:** Process produces at least one status line; continues running until killed by `timeout`.; Process stays alive; watch mode active
- **Exit:** 0
- **Source:** [params.md — interval:: watch mode](../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-7: `interval::0` with `dry::1` combines correctly

- **Given:** Preferred version set in settings.
- **When:** `cm .version.guard interval::0 dry::1`
- **Then:** `[dry-run]` prefixed output; process exits immediately.; dry-run markers present; one-shot exit
- **Exit:** 0
- **Source:** [parameter_interactions.md — dry+interval](../../../../docs/cli/parameter_interactions.md)

---

### EC-8: `interval::` only accepted by `.version.guard`

- **Given:** clean environment
- **When:** `cm .version.install interval::5`
- **Then:** stderr: error about unknown parameter `interval::`; exit code 1.; parameter rejected by non-guard command
- **Exit:** 1
- **Source:** [commands.md — .version.install parameters](../../../../docs/cli/commands.md#command--5-versioninstall)
