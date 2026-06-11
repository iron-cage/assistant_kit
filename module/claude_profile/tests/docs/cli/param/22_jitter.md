# Parameter :: `jitter::`

Edge case tests for the `jitter::` parameter. Tests validate u64 type enforcement, the `jitter <= interval` constraint, default-zero behavior, and conditional validation — the constraint is only enforced when `live::1` is present. Used by `.usage` to add random seconds to the live loop cycle for thunder-herd mitigation.

**Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `jitter::0` — default; exact interval timing (no jitter) | Default |
| EC-2 | `jitter::10` with `live::1 interval::30` — valid (jitter < interval) | Valid Value |
| EC-3 | `jitter::30` with `live::1 interval::30` — valid boundary (jitter == interval) | Boundary Values |
| EC-4 | `jitter::31` with `live::1 interval::30` — rejected (jitter > interval) | Boundary Values |
| EC-5 | Default value is `0` | Default |
| EC-6 | `jitter::70` without `live::1` — accepted (validation skipped) | Conditional Validation |
| EC-7 | `jitter::abc` rejected (type validation) | Type Validation |

## Test Coverage Summary

- Default: 2 tests (EC-1, EC-5)
- Valid Value: 1 test (EC-2)
- Boundary Values: 2 tests (EC-3, EC-4)
- Conditional Validation: 1 test (EC-6)
- Type Validation: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-4 (`live::1` rejects jitter > interval) ↔ EC-6 (`live::0` ignores constraint)

## Test Cases
---

### EC-1: `jitter::0` — default; exact interval timing

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1 jitter::0` (interrupted with Ctrl-C)
- **Then:** Command accepted; live loop cycles on exact `interval::` seconds with no random addition; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it056_jitter_0_explicit_live_accepted`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-2: `jitter::10` with `live::1 interval::30` — valid (jitter < interval)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1 interval::30 jitter::10` (interrupted with Ctrl-C)
- **Then:** Command accepted; each cycle waits 30 + random[0..=10] seconds; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it057_jitter_010_live_accepted`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-3: `jitter::30` with `live::1 interval::30` — valid boundary (jitter == interval)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1 interval::30 jitter::30` (interrupted with Ctrl-C)
- **Then:** Command accepted; jitter equal to interval is the upper boundary and is valid; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it026_live_jitter_equals_interval_accepted`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-4: `jitter::31` with `live::1 interval::30` — rejected (jitter > interval)

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 interval::30 jitter::31`
- **Then:** Exit 1 before any fetch; stderr contains `jitter must not exceed interval`.
- **Exit:** 1
- **Source fn:** `it022_live_jitter_exceeds_interval`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-5: Default value is `0`

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1` (no `jitter::` param; interrupted with Ctrl-C)
- **Then:** Command accepted with default zero jitter; behavior identical to `jitter::0`; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-6: `jitter::70` without `live::1` — accepted (validation skipped)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage jitter::70` (no `live::1`)
- **Then:** Command accepted; single fetch and render; `jitter::` value is ignored when `live::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it028_interval_jitter_ignored_when_not_live`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-7: `jitter::abc` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage jitter::abc`
- **Then:** Exit 1 with type validation error referencing `jitter::`; value must be a non-negative integer.
- **Exit:** 1
- **Source fn:** `it058_jitter_abc_rejected`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
