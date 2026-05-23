# Test: Execution Control Group

Interaction tests for the `dry::`, `force::`, and `interval::` parameter group.
See [003_parameter_groups.md](../../../../docs/cli/003_parameter_groups.md) and [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md).

### Scope

- **Purpose**: Interaction tests for the Execution Control parameter group.
- **Responsibility**: Cross-parameter semantics between `dry::`, `force::`, and `interval::`, precedence rules, and combined behavior.
- **In Scope**: Multi-parameter interactions within the group, dry-wins-force rule, watch loop semantics.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `dry::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set` |
| `force::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill` |
| `interval::` | u64 | 0 | `.version.guard` only |

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `dry::1` always wins over `force::1` | dry+force precedence |
| EC-2 | `dry::0 force::1` → force active (dry::0 means off) | Explicit false |
| IT-3 | `dry::1 force::0` → dry active, force off | Explicit false |
| EC-3 | `dry::1 interval::0` → one-shot dry-run | dry+interval |
| EC-4 | `interval::N` (N>0) starts watch loop | interval>0 |
| IT-6 | `force::1` bypasses idempotency on `.version.guard` | force alone |
| IT-7 | `force::1` on `.processes.kill` → SIGKILL directly | force SIGKILL |
| EC-1 | `dry::1 force::1 interval::0` → dry wins, one-shot | All three |
| EC-2 | `interval::5 dry::1` → watch loop, but each iteration is dry-run | watch+dry |
| EC-3 | `force::1` without `dry::1` → real operation | force alone |
| EC-4 | `dry::0 force::0` explicit → same as both absent | Explicit off |

## Test Coverage Summary

- dry+force precedence: 3 tests (EC-1, EC-2, IT-3)
- dry+interval: 2 tests (EC-3, EC-2)
- interval watch mode: 1 test (EC-4)
- force alone: 2 tests (IT-6, EC-3)
- force SIGKILL: 1 test (IT-7)
- All three combined: 1 test (EC-1)
- Explicit off: 1 test (EC-4)

**Total:** 11 interaction tests

---

### EC-1: `dry::1` wins over `force::1`

- **Given:** clean environment
- **When:** 
- **Then:** Dry-run wins on all three applicable commands
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-2: `dry::0 force::1` → force active

- **Given:** Preference stored; version matches.
- **When:**
  `cm .version.guard dry::0 force::1`
  **Expected:** Real install triggered (bypasses match check).
- **Then:** Force behavior active; dry-run off
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-3: `dry::1 interval::0` → one-shot dry-run

- **Given:** No preference stored.
- **When:**
  `cm .version.guard dry::1 interval::0`
  **Expected:** Exit 0; `[dry-run]` present; process exits immediately.
- **Then:** one-shot; no side effects
- **Exit:** 0
- **Source:** [005_params.md — interval::0](../../../../docs/cli/005_params.md)

---

### EC-4: `interval::N` starts watch loop

- **Given:** No preference stored.
- **When:**
  `timeout 3 cm .version.guard interval::5`
  **Expected:** At least one status line emitted; process kept alive by timeout.
- **Then:** Watch loop active; terminated by `timeout`
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-1: `dry::1 force::1 interval::0` → all three together

- **Given:** No preference stored.
- **When:**
  `cm .version.guard dry::1 force::1 interval::0`
  **Expected:** Exit 0; `[dry-run]` in output; one-shot; no install.
- **Then:** dry wins; one-shot mode; no action
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-2: `interval::5 dry::1` → watch loop with dry-run

- **Given:** No preference stored.
- **When:**
  `timeout 6 cm .version.guard interval::5 dry::1`
  **Expected:** At least one `[dry-run]` line in output; process loops.
- **Then:** Both watch and dry-run active
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-3: `dry::1 force::0` → dry active, force off

- **Given:** clean environment
- **When:** `cm .version.guard dry::1 force::0`
- **Then:** Exit 0; `[dry-run]` present in output; no install performed; force explicitly disabled has no effect beyond absence
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-6: `force::1` bypasses idempotency on `.version.guard`

- **Given:** preferred version already installed and current version matches preferred
- **When:** `cm .version.guard force::1`
- **Then:** Install proceeds despite version already matching; idempotency check skipped; output indicates forced install
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-7: `force::1` on `.processes.kill` → SIGKILL directly

- **Given:** at least one Claude process running
- **When:** `cm .processes.kill force::1`
- **Then:** SIGKILL sent directly to all matching processes without graceful shutdown attempt; processes terminated immediately
- **Exit:** 0
- **Source:** [feature/007_process_management.md](../../../../docs/feature/007_process_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc250_version_install_dry_force_dry_wins` | `integration/cross_cutting_test.rs` |
| `tc251_processes_kill_dry_force_dry_wins` | `integration/cross_cutting_test.rs` |
| `tc252_settings_set_dry_no_write` | `integration/cross_cutting_test.rs` |
| `tc303_version_install_dry_wins_over_force` | `integration/mutation_commands_test.rs` |
| `tc406_guard_dry_force_no_install` | `integration/mutation_commands_test.rs` |
| `tc409_guard_interval_zero_oneshot` | `integration/mutation_commands_test.rs` |
| `tc415_watch_loop_continues_after_install_error` | `integration/mutation_commands_test.rs` |
| `tc493_dry_0_then_1_last_wins_dry_active` | `cli_args_test.rs` |
| `tc494_dry_1_then_0_last_wins_dry_inactive` | `cli_args_test.rs` |
