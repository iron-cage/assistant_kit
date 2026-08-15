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

### EC-1: `jitter::0` (explicit) with `live::1` — accepted, verified via forced exit 2 (not Ctrl-C)

- **Given:** Credential store directory `chmod`-ed to `0o000` (unreadable) before the command runs.
- **When:** `clp .usage live::1 jitter::0` (no explicit `interval::` — relies on the default of 30)
- **Then:** Exits 2 (not 0) — the unreadable store forces failure immediately after the validation guards pass, proving explicit `jitter::0` satisfies the `jitter <= interval` guard without ever entering the interactive loop or observing actual cycle timing. stderr does NOT contain `jitter`. No Ctrl-C is used; store permissions are restored to `0o755` after the assertion. Distinct from `it029` (21_interval.md EC-1), which uses the implicit default (no `jitter::` param at all) — this test exercises the explicit `jitter::0` path.
- **Exit:** 2
- **Source fn:** `it046_jitter_0_explicit_live_accepted` (in `tests/cli/usage_live_test.rs`) — renumbered from `it056` when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-2: `jitter::10` with `live::1 interval::30` — valid (jitter < interval), verified via forced exit 2 (not Ctrl-C)

- **Given:** Credential store directory `chmod`-ed to `0o000` (unreadable) before the command runs.
- **When:** `clp .usage live::1 interval::30 jitter::10`
- **Then:** Exits 2 (not 0) — the unreadable store forces failure immediately after the validation guards pass, proving `jitter::10 <= interval::30` without ever entering the interactive loop or observing an actual wait duration (no "30 + random[0..=10] seconds" timing is measured). stderr does NOT contain `jitter`. No Ctrl-C is used; store permissions are restored to `0o755` after the assertion.
- **Exit:** 2
- **Source fn:** `it047_jitter_10_live_accepted` (in `tests/cli/usage_live_test.rs`) — renumbered from `it057_jitter_010_live_accepted` (also dropped the leading zero: `010`→`10`) when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-3: `jitter::30` with `live::1 interval::30` — valid boundary (jitter == interval), verified via forced exit 2 (not Ctrl-C)

- **Given:** Credential store directory `chmod`-ed to `0o000` (unreadable) before the command runs.
- **When:** `clp .usage live::1 interval::30 jitter::30`
- **Then:** Exits 2 (not 0) — the unreadable store forces failure immediately after the validation guards pass, proving jitter equal to interval (the upper boundary) does not trigger the guard, without ever entering the interactive loop. stderr does NOT contain `jitter`. No Ctrl-C is used; store permissions are restored to `0o755` after the assertion.
- **Exit:** 2
- **Source fn:** `it026_live_jitter_equals_interval_accepted`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-4: `jitter::70` with `live::1 interval::60` — rejected (jitter > interval)

- **Given:** Clean environment (no credentials needed — validation fires before any fetch).
- **When:** `clp .usage live::1 interval::60 jitter::70`
- **Then:** Exits 1 before any fetch; stderr is asserted non-empty (a generic non-empty check), not matched against the literal phrase `jitter must not exceed interval`. The values tested are `interval::60`/`jitter::70`, not `interval::30`/`jitter::31`.
- **Exit:** 1
- **Source fn:** `it022_live_jitter_exceeds_interval`
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
---

### EC-5: Default value is `0` — only verified via help-text listing

- **Given:** None — no account or credential setup.
- **When:** `clp .usage.help`
- **Then:** Exits 0. stdout contains the substrings `live`, `interval`, and `jitter` — confirming the params are documented in help output (AC-32). This test does NOT invoke `.usage live::1` with `jitter::` omitted and does NOT assert zero-jitter runtime behavior. Closest adjacent evidence: `it046_jitter_0_explicit_live_accepted` (EC-1) proves explicit `jitter::0` passes the guard — via the forced exit-2 (unreadable-store) technique, not a runtime timing observation — but exercises the explicit path, not omission.
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
- **Source fn:** `it048_jitter_abc_rejected` (in `tests/cli/usage_live_test.rs`) — renumbered from `it058` when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--22-jitter](../../../../docs/cli/param/022_jitter.md)
