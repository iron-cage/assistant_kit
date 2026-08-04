# Param :: `--gate-poll-secs`

Edge case tests for the `--gate-poll-secs` parameter, which sets the seconds between
polling attempts when the `--max-sessions` concurrency gate is waiting for a slot.

**Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--gate-poll-secs 5` reduces wait between gate attempts | Behavioral |
| EC-2 | `CLR_GATE_POLL_SECS=5` env-var equivalent to `--gate-poll-secs 5` | EnvFallback |
| EC-3 | Default (absent) → 30s interval used | Default |
| EC-4 | `"gate-poll-secs"` JSON key accepted via `--args-file` | JSONConfig |
| EC-5 | CLI flag takes precedence over env var | Precedence |

## Test Coverage Summary

- Behavioral: 1 test (EC-1)
- EnvFallback: 1 test (EC-2)
- Default: 1 test (EC-3)
- JSONConfig: 1 test (EC-4)
- Precedence: 1 test (EC-5)

**Total:** 5 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t_gate_poll_secs_cli_flag_reduces_wait_interval` | `concurrency_gate_test.rs` |
| EC-2 | `t_gate_poll_secs_env_var_equivalent_to_cli_flag` | `concurrency_gate_test.rs` |
| EC-3 | `t_gate_poll_secs_absent_uses_30s_default` | `concurrency_gate_test.rs` |
| EC-4 | `t_gate_poll_secs_json_key_accepted_via_args_file` | `concurrency_gate_test.rs` |
| EC-5 | `t_gate_poll_secs_cli_flag_takes_precedence_over_env_var` | `concurrency_gate_test.rs` |

## Test Cases

---

### EC-1: `--gate-poll-secs 5` reduces wait between gate attempts

- **Given:** one live occupier holds the gate (`--max-sessions 1`); `CLR_GATE_MAX_ATTEMPTS=2`, `CLR_GATE_POLL_SECS` absent
- **When:** `clr --max-sessions 1 --gate-poll-secs 5 --retry-override 0 -p "task"`
- **Then:** gate exhaustion fires after `(2-1) × 5 = 5s` elapsed (not 30s default)
- **Exit:** 1 (gate timed out)
- **Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)

---

### EC-2: `CLR_GATE_POLL_SECS=5` env-var equivalent to `--gate-poll-secs 5`

- **Given:** same fixture as EC-1; `CLR_GATE_POLL_SECS=5` in environment; `--gate-poll-secs` absent from CLI
- **When:** `CLR_GATE_POLL_SECS=5 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** behavior identical to EC-1; gate exhaustion after ~5s
- **Exit:** 1
- **Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)

---

### EC-3: Default (absent) → 30s interval used

- **Given:** clean environment; `--gate-poll-secs` absent; `CLR_GATE_POLL_SECS` absent
- **When:** `clr --dry-run "task"` (dry-run: no real gate)
- **Then:** gate does not trigger in dry-run; parameter is parsed with 30s default
- **Exit:** 0
- **Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)

---

### EC-4: `"gate-poll-secs"` JSON key accepted via `--args-file`

- **Given:** args file containing `{"gate-poll-secs": 5}`; one live occupier; `CLR_GATE_MAX_ATTEMPTS=2`
- **When:** `clr --args-file /tmp/082ec4.json --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** same exhaustion timing as EC-1 (JSON key accepted, 5s interval applied)
- **Exit:** 1
- **Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)

---

### EC-5: CLI flag takes precedence over env var

- **Given:** `CLR_GATE_POLL_SECS=60`; `--gate-poll-secs 5` on CLI; one live occupier; `CLR_GATE_MAX_ATTEMPTS=2`
- **When:** `CLR_GATE_POLL_SECS=60 clr --gate-poll-secs 5 --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** gate exhaustion fires in ~5s (CLI wins over env var)
- **Exit:** 1
- **Source:** [param/082_gate_poll_secs.md](../../../../docs/cli/param/082_gate_poll_secs.md)
