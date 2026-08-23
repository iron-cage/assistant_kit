# Param :: `CLR_REMAINING_TIMEOUT_SECS`

Edge case tests for the `CLR_REMAINING_TIMEOUT_SECS` env var, which clamps the concurrency
gate's effective attempt count to `floor(remaining_secs / poll_secs).max(1)` so gate-wait
polling does not outlive a wrapping job-runner deadline.

**Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `CLR_REMAINING_TIMEOUT_SECS=2`, `CLR_GATE_POLL_SECS=1`, `CLR_GATE_MAX_ATTEMPTS=1000` → gate exhausts after 2 attempts with "budget" diagnostic | Behavioral |
| EC-2 | `CLR_REMAINING_TIMEOUT_SECS=1`, `CLR_GATE_POLL_SECS=30` → `.max(1)` floor: at least 1 attempt, instant budget-exhaustion | BoundaryFloor |
| EC-3 | Absent → no clamping; normal `CLR_GATE_MAX_ATTEMPTS` ceiling applies; announced `off (…unset)` | Default |
| EC-4 | Non-numeric value → resolves to `None`; no crash; normal ceiling applies; announced `off (…unparseable)` with raw value | Validation |
| EC-5 | Unset vs unparseable vs set-but-nonlimiting → three distinct `gate-deadline` announcements, mutually distinguishable (BUG-481) | Validation |
| EC-6 | Empty string and negative → announced off-unparseable; `"0"` → engaged with `.max(1)` one-attempt floor, budget path (BUG-481) | BoundaryFloor |
| EC-7 | `CLR_GATE_POLL_SECS=0` + numeric budget → no divide-by-zero; divisor floored to 1; announced nonlimiting (BUG-481) | Validation |

## Test Coverage Summary

- Behavioral: 1 test (EC-1)
- BoundaryFloor: 2 tests (EC-2, EC-6)
- Default: 1 test (EC-3)
- Validation: 3 tests (EC-4, EC-5, EC-7)

**Total:** 7 edge cases

## Implementation Notes

<!-- BUG-481 — fixed: File column corrected to concurrency_gate_ext3_test.rs (all rows; previously routed to concurrency_gate_test.rs, where none of the implementations live) and EC-5/6/7 registered for the resolution-announcement contract. All 7 rows later moved again, unchanged, to concurrency_gate_deadline_test.rs when ext3 was split at the 1500-line threshold -->
| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t35_remaining_timeout_budget_clamps_gate_attempts` | `concurrency_gate_deadline_test.rs` |
| EC-2 | `t36_remaining_timeout_below_poll_interval_still_makes_one_attempt` | `concurrency_gate_deadline_test.rs` |
| EC-3 | `t_gate_remaining_timeout_absent_uses_normal_max_attempts` | `concurrency_gate_deadline_test.rs` |
| EC-4 | `t_gate_remaining_timeout_non_numeric_resolves_to_none` | `concurrency_gate_deadline_test.rs` |
| EC-5 | `t39_deadline_resolution_states_announced_and_distinguishable` | `concurrency_gate_deadline_test.rs` |
| EC-6 | `t40_deadline_boundary_inputs_resolve_and_announce` | `concurrency_gate_deadline_test.rs` |
| EC-7 | `t41_poll_secs_zero_with_numeric_budget_does_not_panic` | `concurrency_gate_deadline_test.rs` |

## Test Cases

---

### EC-1: `CLR_REMAINING_TIMEOUT_SECS=2`, `CLR_GATE_POLL_SECS=1` → gate clamps to 2 attempts with "budget" diagnostic

- **Given:** one live occupier holds the gate (`--max-sessions 1`); `CLR_GATE_POLL_SECS=1`; `CLR_GATE_MAX_ATTEMPTS=1000`; `CLR_REMAINING_TIMEOUT_SECS=2`
- **When:** `CLR_REMAINING_TIMEOUT_SECS=2 CLR_GATE_POLL_SECS=1 CLR_GATE_MAX_ATTEMPTS=1000 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** gate exhausts after exactly 2 attempts (floor(2/1)=2; not 1000); stderr contains `"budget"`; stderr does NOT contain `"session gate timed out"` (the two exhaustion paths are distinct)
- **Exit:** 1
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

---

### EC-2: budget below one poll interval → `.max(1)` floor makes 1 attempt (no sleep)

- **Given:** same fixture; `CLR_GATE_POLL_SECS=30`; `CLR_REMAINING_TIMEOUT_SECS=1`
- **When:** `CLR_REMAINING_TIMEOUT_SECS=1 CLR_GATE_POLL_SECS=30 CLR_GATE_MAX_ATTEMPTS=1000 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** floor(1/30)=0 → .max(1)=1; gate makes exactly 1 attempt and immediately exhausts (no 30s sleep before exhaustion); stderr contains `"budget"`
- **Exit:** 1
- **Note:** the test completes in < 1s because sleep follows the exhaustion check, not precedes it
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

---

### EC-3: absent → no budget clamp; gate uses normal `CLR_GATE_MAX_ATTEMPTS` ceiling

- **Given:** `CLR_REMAINING_TIMEOUT_SECS` not set; `CLR_GATE_MAX_ATTEMPTS=2`; `CLR_GATE_POLL_SECS=1`; one live occupier
- **When:** `clr --max-sessions 1 --retry-override 0 -p "task"` (no `CLR_REMAINING_TIMEOUT_SECS`)
- **Then:** gate exhausts after 2 attempts (the normal `CLR_GATE_MAX_ATTEMPTS=2` ceiling); stderr contains `"session gate timed out"` (not `"budget"`) and the announcement `off (CLR_REMAINING_TIMEOUT_SECS unset)` (BUG-481)
- **Exit:** 1
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

---

### EC-4: non-numeric value → resolves to `None`; no crash; normal ceiling applies

- **Given:** `CLR_REMAINING_TIMEOUT_SECS=notanumber`; `CLR_GATE_MAX_ATTEMPTS=2`; `CLR_GATE_POLL_SECS=1`; one live occupier
- **When:** `CLR_REMAINING_TIMEOUT_SECS=notanumber CLR_GATE_MAX_ATTEMPTS=2 CLR_GATE_POLL_SECS=1 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** invalid value resolves to `None` (feature off); gate exhausts normally after 2 attempts; no crash; stderr contains `"session gate timed out"` (not `"budget"`) and the announcement `off (CLR_REMAINING_TIMEOUT_SECS="notanumber" unparseable)` — misconfiguration is distinguishable from non-configuration (BUG-481)
- **Exit:** 1
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

---

### EC-5: unset / unparseable / set-but-nonlimiting → three mutually distinguishable announcements

- **Given:** a denied gate fixture (sole slot held by a live owner, empty census); `CLR_GATE_POLL_SECS=1`; `CLR_GATE_MAX_ATTEMPTS=2`; `CLR_GATE_STALE_SECS` removed; three legs: env var unset, `=notanumber`, `=30000` (floor(30000/1)=30000 ≥ 2, so the strict-`<` clamp never engages)
- **When:** one `clr -p --max-sessions 1 --retry-override 0` run per leg, stderr captured
- **Then:** each leg's stderr carries a `gate-deadline` line — `off (CLR_REMAINING_TIMEOUT_SECS unset)` / `off (CLR_REMAINING_TIMEOUT_SECS="notanumber" unparseable)` / `nonlimiting (30000s covers all 2 attempts)` — the three lines are pairwise distinct, the unset leg's line also names `stale-reclaim off`, and every leg still exhausts via `"session gate timed out"` (resolution semantics stay feature-off)
- **Exit:** non-zero in every leg
- **Note:** `bug_reproducer(BUG-481)` — pre-fix, all three legs produced byte-identical output modulo timestamps (the MRE's diff assertion); empirically confirmed to fail pre-fix and pass post-fix
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md) resolution-announcement Note

---

### EC-6: empty / negative / `"0"` boundary inputs resolve and announce deterministically

- **Given:** same denied fixture as EC-5; legs `=""`, `="-5"`, `="0"`
- **When:** one run per leg, stderr captured
- **Then:** `""` and `"-5"` fail the u64 parse and announce `off (CLR_REMAINING_TIMEOUT_SECS="" unparseable)` / `off (CLR_REMAINING_TIMEOUT_SECS="-5" unparseable)`, exhausting via `"session gate timed out"`; `"0"` parses, engages with the `.max(1)` one-attempt floor, announces `engaged (0s clamps to 1 of 2 attempts)`, and exhausts via `"gate-wait budget exhausted"`
- **Exit:** non-zero in every leg
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md) `.max(1)` floor Note

---

### EC-7: `CLR_GATE_POLL_SECS=0` with a numeric budget → no divide-by-zero

- **Given:** same denied fixture; `CLR_GATE_POLL_SECS=0`; `CLR_GATE_MAX_ATTEMPTS=2`; `CLR_REMAINING_TIMEOUT_SECS=10`
- **When:** `clr -p --max-sessions 1 --retry-override 0` runs, stderr captured
- **Then:** no panic (pre-fix: integer divide-by-zero in `effective_gate_attempts` — the divisor path only runs when the env var parses numeric, so the crash needed both knobs set); quotient divisor floored to 1 → floor(10/1)=10 ≥ 2 → announced `nonlimiting (10s covers all 2 attempts)`; gate exhausts via `"session gate timed out"`
- **Exit:** non-zero, not a panic exit
- **Note:** `bug_reproducer(BUG-481)` — empirically confirmed to fail pre-fix (child panicked at the division) and pass post-fix
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md) `.max(1)` floor Note
