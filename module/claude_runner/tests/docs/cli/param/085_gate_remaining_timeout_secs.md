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
| EC-3 | Absent → no clamping; normal `CLR_GATE_MAX_ATTEMPTS` ceiling applies | Default |
| EC-4 | Non-numeric value → resolves to `None`; no crash; normal ceiling applies | Validation |

## Test Coverage Summary

- Behavioral: 1 test (EC-1)
- BoundaryFloor: 1 test (EC-2)
- Default: 1 test (EC-3)
- Validation: 1 test (EC-4)

**Total:** 4 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t35_remaining_timeout_budget_clamps_gate_attempts` | `concurrency_gate_test.rs` |
| EC-2 | `t36_remaining_timeout_below_poll_interval_still_makes_one_attempt` | `concurrency_gate_test.rs` |
| EC-3 | `t_gate_remaining_timeout_absent_uses_normal_max_attempts` | `concurrency_gate_test.rs` |
| EC-4 | `t_gate_remaining_timeout_non_numeric_resolves_to_none` | `concurrency_gate_test.rs` |

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
- **Then:** gate exhausts after 2 attempts (the normal `CLR_GATE_MAX_ATTEMPTS=2` ceiling); stderr contains `"session gate timed out"` (not `"budget"`)
- **Exit:** 1
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)

---

### EC-4: non-numeric value → resolves to `None`; no crash; normal ceiling applies

- **Given:** `CLR_REMAINING_TIMEOUT_SECS=notanumber`; `CLR_GATE_MAX_ATTEMPTS=2`; `CLR_GATE_POLL_SECS=1`; one live occupier
- **When:** `CLR_REMAINING_TIMEOUT_SECS=notanumber CLR_GATE_MAX_ATTEMPTS=2 CLR_GATE_POLL_SECS=1 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** invalid value silently resolves to `None` (feature off); gate exhausts normally after 2 attempts; no crash; stderr contains `"session gate timed out"` (not `"budget"`)
- **Exit:** 1
- **Source:** [param/085_gate_remaining_timeout_secs.md](../../../../docs/cli/param/085_gate_remaining_timeout_secs.md)
