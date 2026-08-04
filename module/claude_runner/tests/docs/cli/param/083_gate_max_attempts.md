# Param :: `--gate-max-attempts`

Edge case tests for the `--gate-max-attempts` parameter, which sets the maximum number of
admission attempts before the `--max-sessions` concurrency gate declares exhaustion.

**Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--gate-max-attempts 2` → gate exhausts after 2 attempts | Behavioral |
| EC-2 | `CLR_GATE_MAX_ATTEMPTS=2` env-var equivalent to `--gate-max-attempts 2` | EnvFallback |
| EC-3 | Default (absent) → 1000 attempts used | Default |
| EC-4 | `"gate-max-attempts"` JSON key accepted via `--args-file` | JSONConfig |
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
| EC-1 | `t_gate_max_attempts_cli_flag_exhausts_after_n_attempts` | `concurrency_gate_test.rs` |
| EC-2 | `t_gate_max_attempts_env_var_equivalent_to_cli_flag` | `concurrency_gate_test.rs` |
| EC-3 | `t_gate_max_attempts_absent_uses_1000_default` | `concurrency_gate_test.rs` |
| EC-4 | `t_gate_max_attempts_json_key_accepted_via_args_file` | `concurrency_gate_test.rs` |
| EC-5 | `t_gate_max_attempts_cli_flag_takes_precedence_over_env_var` | `concurrency_gate_test.rs` |

## Test Cases

---

### EC-1: `--gate-max-attempts 2` → gate exhausts after 2 attempts

- **Given:** one live occupier holds the gate (`--max-sessions 1`); `CLR_GATE_POLL_SECS=1`
- **When:** `clr --max-sessions 1 --gate-max-attempts 2 --retry-override 0 -p "task"`
- **Then:** gate exhaustion fires after exactly 2 attempts (not 1000); exit 1 with `session gate timed out` message
- **Exit:** 1
- **Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)

---

### EC-2: `CLR_GATE_MAX_ATTEMPTS=2` env-var equivalent to `--gate-max-attempts 2`

- **Given:** same fixture as EC-1; `CLR_GATE_MAX_ATTEMPTS=2`; `--gate-max-attempts` absent from CLI
- **When:** `CLR_GATE_MAX_ATTEMPTS=2 clr --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** behavior identical to EC-1; exits after 2 attempts
- **Exit:** 1
- **Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)

---

### EC-3: Default (absent) → 1000 attempts used

- **Given:** clean environment; `--gate-max-attempts` absent; `CLR_GATE_MAX_ATTEMPTS` absent
- **When:** `clr --dry-run "task"` (dry-run skips gate)
- **Then:** gate does not trigger in dry-run; parameter parsed with default 1000
- **Exit:** 0
- **Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)

---

### EC-4: `"gate-max-attempts"` JSON key accepted via `--args-file`

- **Given:** args file containing `{"gate-max-attempts": 2}`; one live occupier; `CLR_GATE_POLL_SECS=1`
- **When:** `clr --args-file /tmp/083ec4.json --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** same behavior as EC-1 (JSON key accepted, 2-attempt limit applied)
- **Exit:** 1
- **Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)

---

### EC-5: CLI flag takes precedence over env var

- **Given:** `CLR_GATE_MAX_ATTEMPTS=100`; `--gate-max-attempts 2` on CLI; one live occupier; `CLR_GATE_POLL_SECS=1`
- **When:** `CLR_GATE_MAX_ATTEMPTS=100 clr --gate-max-attempts 2 --max-sessions 1 --retry-override 0 -p "task"`
- **Then:** gate exhausts after 2 attempts (CLI flag wins; not 100 from env var)
- **Exit:** 1
- **Source:** [param/083_gate_max_attempts.md](../../../../docs/cli/param/083_gate_max_attempts.md)
