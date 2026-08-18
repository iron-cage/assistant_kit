# Param :: `--gate-stale-secs`

Edge case tests for the `--gate-stale-secs` parameter, which sets the staleness threshold
for reclaiming a live-but-stalled slot in the `--max-sessions` concurrency gate. When
unset (default), live slot owners are never reclaimed regardless of elapsed time.

**Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default (absent) → live owner never reclaimed | Default |
| EC-2 | `--gate-stale-secs N` → stalled slot reclaimed after N seconds | Behavioral |
| EC-3 | `CLR_GATE_STALE_SECS=N` env-var equivalent | EnvFallback |
| EC-4 | `"gate-stale-secs"` JSON key accepted via `--args-file` | JSONConfig |
| EC-5 | CLI flag takes precedence over env var | Precedence |
| EC-6 | Invalid value (non-numeric) → resolves to `None` (feature off) | Validation |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Behavioral: 1 test (EC-2)
- EnvFallback: 1 test (EC-3)
- JSONConfig: 1 test (EC-4)
- Precedence: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t_gate_stale_secs_absent_live_owner_never_reclaimed` | `concurrency_gate_ext3_test.rs` |
| EC-2 | `t_gate_stale_secs_cli_flag_reclaims_stale_slot` | `concurrency_gate_ext3_test.rs` |
| EC-3 | `t_gate_stale_secs_env_var_reclaims_stale_slot` | `concurrency_gate_ext3_test.rs` |
| EC-4 | `t_gate_stale_secs_json_key_accepted_via_args_file` | `concurrency_gate_ext3_test.rs` |
| EC-5 | `t_gate_stale_secs_cli_flag_takes_precedence_over_env_var` | `concurrency_gate_ext3_test.rs` |
| EC-6 | `t_gate_stale_secs_invalid_value_resolves_to_none` | `concurrency_gate_ext3_test.rs` |

## Test Cases

---

### EC-1: Default (absent) → live owner never reclaimed

- **Given:** one live occupier holds the gate; `CLR_GATE_STALE_SECS` absent; `--gate-stale-secs` absent; `CLR_GATE_MAX_ATTEMPTS=2`, `CLR_GATE_POLL_SECS=1`
- **When:** `clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** gate exhausts after 2 attempts (no reclaim despite occupier being "stale"); feature is off by default
- **Exit:** 1
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

---

### EC-2: `--gate-stale-secs N` → stalled slot reclaimed after N seconds

- **Given:** one stalled occupier (gate state file deliberately not updated for >N seconds); `CLR_GATE_MAX_ATTEMPTS=5`, `CLR_GATE_POLL_SECS=1`
- **When:** `clr --max-sessions 1 --gate-stale-secs 1 -p "task"`
- **Then:** gate admits the second invocation after reclaiming the stale slot; success
- **Exit:** 0
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

---

### EC-3: `CLR_GATE_STALE_SECS=N` env-var equivalent

- **Given:** same stalled-occupier fixture as EC-2; `CLR_GATE_STALE_SECS=1`; `--gate-stale-secs` absent
- **When:** `CLR_GATE_STALE_SECS=1 clr --max-sessions 1 -p "task"`
- **Then:** behavior identical to EC-2; stale slot reclaimed via env var
- **Exit:** 0
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

---

### EC-4: `"gate-stale-secs"` JSON key accepted via `--args-file`

- **Given:** args file containing `{"gate-stale-secs": 1}`; same stalled-occupier fixture as EC-2
- **When:** `clr --args-file /tmp/084ec4.json --max-sessions 1 -p "task"`
- **Then:** stale slot reclaimed (JSON key accepted, feature active with 1s threshold)
- **Exit:** 0
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

---

### EC-5: CLI flag takes precedence over env var

- **Given:** `CLR_GATE_STALE_SECS=0`; `--gate-stale-secs 1` on CLI; stalled-occupier fixture
- **When:** `CLR_GATE_STALE_SECS=0 clr --gate-stale-secs 1 --max-sessions 1 -p "task"`
- **Then:** CLI value (1s) wins; stale slot reclaimed (not the 0s env var, which is aggressive but the same direction — verify the threshold applied is 1s not 0s via trace output)
- **Exit:** 0
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)

---

### EC-6: Invalid value (non-numeric) → resolves to `None` (feature off)

- **Given:** `CLR_GATE_STALE_SECS=notanumber`; live occupier holds slot; `CLR_GATE_MAX_ATTEMPTS=2`, `CLR_GATE_POLL_SECS=1`
- **When:** `CLR_GATE_STALE_SECS=notanumber clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** gate exhausts after 2 attempts (invalid value silently resolves to `None`; live owner not reclaimed); no crash
- **Exit:** 1
- **Source:** [param/084_gate_stale_secs.md](../../../../docs/cli/param/084_gate_stale_secs.md)
