# Parameter :: `interval::`

Edge case tests for the `interval::` parameter. Tests validate u64 type enforcement, minimum boundary (≥ 30), and conditional validation — the constraint is only enforced when `live::1` is present. Used by `.usage` to set seconds between full refresh cycles in live monitor mode.

**Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)

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

### EC-1: Default interval (30, omitted) with `live::1` — minimum boundary accepted, verified via forced exit 2 (not Ctrl-C)

- **Given:** Credential store directory `chmod`-ed to `0o000` (unreadable) before the command runs. No explicit `interval::` is passed — no test in the suite passes the literal `interval::30`.
- **When:** `clp .usage live::1` (relies on the default value of 30, which is also the minimum)
- **Then:** Exits 2 (not 0) — the unreadable store forces `execute_live_mode()` to fail immediately after the validation guards pass, proving the default/minimum interval (30) satisfies the `>= 30` guard without ever entering the interactive loop. stderr does NOT contain `interval`. No Ctrl-C is used; store permissions are restored to `0o755` after the assertion.
- **Exit:** 2
- **Source fn:** `it029_live_default_interval_accepted`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
---

### EC-2: `interval::5` with `live::1 jitter::0` — rejected (below minimum)

- **Given:** Clean environment (no credentials needed — validation fires before any fetch).
- **When:** `clp .usage live::1 interval::5 jitter::0`
- **Then:** Exits 1 before any fetch. stderr contains the substring `30` (a generic substring check that the minimum value is mentioned) — not a match against the literal phrase `interval must be >= 30`. The below-minimum value tested is `5`, not `29`.
- **Exit:** 1
- **Source fn:** `it023_live_interval_below_minimum`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
---

### EC-3: `interval::60` with `live::1` — non-default value accepted, verified via forced exit 2 (not Ctrl-C)

- **Given:** Credential store directory `chmod`-ed to `0o000` (unreadable) before the command runs.
- **When:** `clp .usage live::1 interval::60`
- **Then:** Exits 2 (not 0) — the unreadable store forces failure immediately after the validation guards pass, proving `interval::60` satisfies the `>= 30` guard without ever entering the interactive loop. stderr does NOT contain `interval`. No Ctrl-C is used; store permissions are restored to `0o755` after the assertion.
- **Exit:** 2
- **Source fn:** `it045_interval_60_live_accepted` (in `tests/cli/usage_live_test.rs`) — renumbered from `it055` when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
---

### EC-4: Default value is `30` — only verified via help-text listing

- **Given:** None — no account or credential setup.
- **When:** `clp .usage.help`
- **Then:** Exits 0. stdout contains the substrings `live`, `interval`, and `jitter` — confirming the params are documented in help output (AC-32). This test does NOT invoke `.usage live::1` with `interval::` omitted and does NOT assert a 30-second cycle at runtime. Closest adjacent evidence: `it029_live_default_interval_accepted` (EC-1) exercises the true omitted-`interval::` default path, proving the default (30) passes the `>= 30` guard — via the forced exit-2 (unreadable-store) technique, not a runtime cycle-timing observation.
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
---

### EC-5: `interval::29` without `live::1` — accepted (validation skipped)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage interval::29` (no `live::1`)
- **Then:** Command accepted; single fetch and render; `interval::` value is ignored when `live::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it028_interval_jitter_ignored_when_not_live`
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
---

### EC-6: `interval::abc` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage interval::abc`
- **Then:** Exit 1 with type validation error referencing `interval::`; value must be a non-negative integer.
- **Exit:** 1
- **Source fn:** `it044_interval_abc_rejected` (in `tests/cli/usage_live_test.rs`) — renumbered from `it054` when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--21-interval](../../../../docs/cli/param/021_interval.md)
