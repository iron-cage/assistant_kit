# Test: Execution Control Group

Interaction tests for the `dry::`, `force::`, and `interval::` parameter group.
See [003_parameter_groups.md](../../../../docs/cli/003_parameter_groups.md) and [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md).

### Scope

- **Purpose**: Interaction tests for the Execution Control parameter group.
- **Responsibility**: Cross-parameter semantics between `dry::`, `force::`, and `interval::`, precedence rules, and combined behavior.
- **Commands:** `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set`
- **In Scope**: Multi-parameter interactions within the group, dry-wins-force rule, watch loop semantics.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `dry::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set` |
| `force::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill` |
| `interval::` | u64 | 0 | `.version.guard` only |

## Behavioral Divergence Pair

Two valid invocations produce distinct behavior on the same command:

- **Input A:** `cm .version.install dry::1 force::1` → `[dry-run]` prefix; no install (dry wins over force)
- **Input B:** `cm .version.install dry::0 force::1` → no dry-run prefix; force install proceeds

Both are valid invocations; the dry-run flag presence in output differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `dry::1 force::1` → dry wins, `[dry-run]` prefix | dry+force precedence |
| CC-2 | `dry::0 force::1` → force active (dry::0 means off) | Explicit false |
| CC-3 | `dry::1 force::0` → dry active, force off | Explicit false |
| CC-4 | `dry::1 interval::0` → one-shot dry-run | dry+interval |
| CC-5 | `interval::N` (N>0) starts watch loop | interval>0 |
| CC-6 | `force::1` bypasses idempotency on `.version.guard` | force alone |
| CC-7 | `force::1` on `.processes.kill` → SIGKILL directly | force SIGKILL |
| CC-8 | `dry::1 force::1 interval::0` → dry wins, one-shot | All three |
| CC-9 | `interval::5 dry::1` → watch loop, each iteration dry-run | watch+dry |
| CC-10 | `force::1` without `dry::1` → real operation | force alone |
| CC-11 | `dry::0 force::0` explicit → same as both absent | Explicit off |

## Test Coverage Summary

- dry+force precedence: 3 tests (CC-1, CC-2, CC-3)
- dry+interval: 2 tests (CC-4, CC-9)
- interval watch mode: 1 test (CC-5)
- force alone: 2 tests (CC-6, CC-10)
- force SIGKILL: 1 test (CC-7)
- All three combined: 1 test (CC-8)
- Explicit off: 1 test (CC-11)

**Total:** 11 interaction tests

---

### CC-1: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:** `cm .version.install dry::1 force::1`
- **Then:** `[dry-run]` prefix present; no install executed; dry-run wins on all applicable commands
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-2: `dry::0 force::1` → force active

- **Given:** preference stored; version matches
- **When:** `cm .version.guard dry::0 force::1`
- **Then:** real install triggered (bypasses match check); no `[dry-run]` prefix; force behavior active
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-3: `dry::1 force::0` → dry active, force off

- **Given:** clean environment
- **When:** `cm .version.guard dry::1 force::0`
- **Then:** exit 0; `[dry-run]` present in output; no install performed; explicitly-disabled force has no effect
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-4: `dry::1 interval::0` → one-shot dry-run

- **Given:** no preference stored
- **When:** `cm .version.guard dry::1 interval::0`
- **Then:** exit 0; `[dry-run]` present; process exits immediately (one-shot mode)
- **Exit:** 0
- **Source:** [005_params.md — interval::0](../../../../docs/cli/005_params.md)

---

### CC-5: `interval::N` (N>0) starts watch loop

- **Given:** no preference stored
- **When:** `timeout 3 cm .version.guard interval::5`
- **Then:** at least one status line emitted; process stays alive until killed by timeout
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### CC-6: `force::1` bypasses idempotency on `.version.guard`

- **Given:** preferred version already installed and current version matches preferred
- **When:** `cm .version.guard force::1`
- **Then:** install proceeds despite version already matching; idempotency check skipped
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### CC-7: `force::1` on `.processes.kill` → SIGKILL directly

- **Given:** at least one Claude process running
- **When:** `cm .processes.kill force::1`
- **Then:** SIGKILL sent directly to all matching processes without graceful shutdown attempt
- **Exit:** 0
- **Source:** [feature/002_process_lifecycle.md — Kill sequence force mode](../../../../docs/feature/002_process_lifecycle.md)

---

### CC-8: `dry::1 force::1 interval::0` → all three together

- **Given:** no preference stored
- **When:** `cm .version.guard dry::1 force::1 interval::0`
- **Then:** exit 0; `[dry-run]` in output; one-shot mode; no install executed
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-9: `interval::5 dry::1` → watch loop with dry-run

- **Given:** no preference stored
- **When:** `timeout 6 cm .version.guard interval::5 dry::1`
- **Then:** at least one `[dry-run]` line in output; process loops with dry-run on each iteration
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### CC-10: `force::1` without `dry::1` → real operation

- **Given:** clean environment
- **When:** `cm .version.install force::1 dry::1` with `dry::` intentionally absent → `cm .version.install force::1`
- **Then:** install proceeds as real operation; no `[dry-run]` prefix; force bypass active
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### CC-11: `dry::0 force::0` explicit → same as both absent

- **Given:** clean environment
- **When:** `cm .version.install dry::0 force::0 dry::1` — no: `cm .version.guard dry::0 force::0`
- **Then:** behavior identical to `cm .version.guard` with no flags; explicit zeros equal absent
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

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
