# Parameter :: `interval::`

Edge case tests for the `interval::` parameter. Tests validate u64 type enforcement, minimum boundary (≥ 30), and conditional validation — the constraint is only enforced when `live::1` is present. Used by `.usage` to set seconds between full refresh cycles in live monitor mode.

**Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `interval::30` with `live::1` — minimum boundary accepted | Boundary Values |
| EC-2 | `interval::29` with `live::1` — rejected (below minimum) | Boundary Values |
| EC-3 | `interval::60` with `live::1` — non-default value accepted | Valid Value |
| EC-4 | Default value is `30` | Default |
| EC-5 | `interval::29` without `live::1` — accepted (validation skipped) | Conditional Validation |
| EC-6 | `interval::abc` rejected (type validation) | Type Validation |

## Test Coverage Summary

- Boundary Values: 2 tests (EC-1, EC-2)
- Valid Value: 1 test (EC-3)
- Default: 1 test (EC-4)
- Conditional Validation: 1 test (EC-5)
- Type Validation: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-2 (`live::1` rejects below-minimum) ↔ EC-5 (`live::0` ignores constraint)

## Test Cases
---

### EC-1: `interval::30` with `live::1` — minimum boundary accepted

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1 interval::30` (interrupted with Ctrl-C)
- **Then:** Command accepted; enters live loop with 30-second cycle; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it29_live_default_interval_accepted`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
---

### EC-2: `interval::29` with `live::1` — rejected (below minimum)

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 interval::29`
- **Then:** Exit 1 before any fetch; stderr contains `interval must be >= 30`.
- **Exit:** 1
- **Source fn:** `it23_live_interval_below_minimum`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
---

### EC-3: `interval::60` with `live::1` — non-default value accepted

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1 interval::60` (interrupted with Ctrl-C)
- **Then:** Command accepted; enters live loop with 60-second cycle; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it45_interval_60_live_accepted`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
---

### EC-4: Default value is `30`

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1` (no `interval::` param; interrupted with Ctrl-C)
- **Then:** Command accepted with default 30-second cycle; behavior identical to `interval::30`; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it31_usage_help_shows_live_params`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
---

### EC-5: `interval::29` without `live::1` — accepted (validation skipped)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage interval::29` (no `live::1`)
- **Then:** Command accepted; single fetch and render; `interval::` value is ignored when `live::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it28_interval_jitter_ignored_when_not_live`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
---

### EC-6: `interval::abc` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage interval::abc`
- **Then:** Exit 1 with type validation error referencing `interval::`; value must be a non-negative integer.
- **Exit:** 1
- **Source fn:** `it44_interval_abc_rejected`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/21_interval.md)
