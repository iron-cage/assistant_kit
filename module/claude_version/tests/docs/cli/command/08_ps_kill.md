# Test: `.ps.kill`

### Scope

- **Purpose**: Integration test cases for the `.ps.kill` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for process termination.
- **In Scope**: SIGTERM/SIGKILL sequence, targeted kill via pid::, force mode, dry-run, post-kill verification.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.ps.kill` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: real kill | Default behavior |
| 0 | Explicit: real kill | Explicit false |
| 1 | Preview only: no kill | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 2: `force::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: SIGTERM → wait → SIGKILL | Default behavior |
| 0 | Explicit: SIGTERM sequence | Explicit false |
| 1 | SIGKILL directly (no SIGTERM) | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 3: Interaction: `dry::1` vs `force::1`

| Combination | Behavior |
|-------------|----------|
| `dry::1` alone | Preview: "no active processes" or "[dry-run] would kill N" |
| `force::1` alone | Real SIGKILL |
| `dry::1 force::1` | dry wins: preview only, no kill |

### Factor 4: Active processes (Environmental — /proc global state)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No claude processes | No-op |
| one or more | Processes found | Kill sequence |

**Note:** Tests cannot control /proc state. All automated tests must handle both empty
and non-empty /proc results gracefully.

### Factor 5: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 6: `v::` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: labeled output | Default behavior |
| 0 | Bare count / minimal output | Compact |
| 1 | Labeled message | Labeled |

### Factor 7: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: text output | Default behavior |
| `text` | Human-readable text | Valid |
| `json` | Machine-readable JSON | Valid |
| `JSON` | Wrong case | Invalid: exit 1 |

### Factor 8: `pid::` (u64, optional, absent = bulk mode)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Bulk kill: all claude processes | Default behavior |
| valid claude PID | Targeted kill of one process | Targeted mode |
| non-claude PID | PID exists but is not claude | Invalid: exit 1 |
| nonexistent PID | PID not in /proc | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | No processes → "no active processes", exit 0 | P | 0 | F1=absent, F4=none | [mutation_ps_kill_test.rs] |
| IT-2 | `dry::1` no processes → "no active processes" | P | 0 | F1=1, F4=none | [mutation_ps_kill_test.rs] |
| IT-3 | `dry::1 force::1` no processes → "no active processes" | P | 0 | F1=1, F2=1, F3, F4=none | [mutation_ps_kill_test.rs] |
| IT-4 | `v::0` → accepted, exit 0 | P | 0 | F6=0 | [mutation_ps_kill_test.rs] |
| IT-6 | Source-level AF: `let _ = send_sig` absent from commands/process.rs | P | 0 | — | [mutation_ps_kill_test.rs] |
| IT-7 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F1=1, F7=json | [mutation_ps_kill_test.rs] |
| IT-11 | `pid::1 dry::1` → exit 1 (non-claude PID) or dry preview | P/N | 0/1 | F8=non-claude | [mutation_ps_kill_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-5 | `format::JSON` (uppercase) → exit 1 | N | 1 | F7=JSON | [mutation_ps_kill_test.rs] |
| IT-8 | `bogus::x` → exit 1 | N | 1 | F5=present | [mutation_ps_kill_test.rs] |
| IT-9 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F1=2 | [mutation_ps_kill_test.rs] |
| IT-10 | `force::2` → exit 1, out-of-range boolean | N | 1 | F2=2 | [mutation_ps_kill_test.rs] |
| IT-12 | `pid::99999999` → exit 1, nonexistent PID | N | 1 | F8=nonexistent | [mutation_ps_kill_test.rs] |
| IT-13 | `pid::abc` → exit 1, non-integer | N | 1 | F8=abc | [mutation_ps_kill_test.rs] |

### Summary

- **Total:** 13 tests (7 positive/mixed, 6 negative)
- **Negative ratio:** 46% ✅ (≥40%)
- **TC range:** IT-1 to IT-7, IT-8 to IT-13

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (kill or no-op) | IT-1, IT-2, IT-3, IT-4, IT-6, IT-7 |
| 1 | Invalid arguments or non-claude PID | IT-5, IT-8 through IT-10, IT-12, IT-13 |
| 2 | Kill verification failure (post-kill survivors) | Manual only (FR-09) |

### Kill Sequence Coverage

| Scenario | Coverage |
|----------|---------|
| No processes (no-op) | IT-1 |
| `dry::1` no processes | IT-2 |
| `dry::1 force::1` (dry wins) | IT-3 |
| Targeted kill with `pid::` | IT-11 (dry preview) |
| Non-claude PID rejected | IT-11, IT-12 |
| Real SIGTERM sequence (with processes) | Manual (requires live processes) |
| `force::1` SIGKILL (with processes) | Manual (requires live processes) |

IT-1 through IT-3 cover the "no processes" path. Real kill sequences require
live claude processes and are manual-only tests.

---

## Test Case Details

---

### IT-1: No processes → "no active processes"

- **Given:** No claude processes in /proc (may not be guaranteed).
- **When:**
  `clv .ps.kill`
  **Expected:** Exit 0; stdout contains "no active processes" or similar.
- **Then:** exit 0; stdout contains "no active processes" message or kill completion summary; either outcome accepted due to /proc global state
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-2: `dry::1` no processes

- **Given:** clean environment
- **When:**
  `clv .ps.kill dry::1`
  **Expected:** Exit 0; appropriate message.
- **Then:** exit 0; stdout contains "[dry-run]" indicator or "no active processes" message; no kill executed; stderr is empty
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-3: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:**
  `clv .ps.kill dry::1 force::1`
  **Expected:** Exit 0; no kill executed.
- **Then:** exit 0; stdout contains dry-run preview; no kill executed even though force::1 is present (dry wins); stderr is empty
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-4: `v::0` → accepted, exit 0

- **Given:** clean environment
- **When:**
  `clv .ps.kill v::0`
  **Expected:** Exit 0; output produced (either "no active processes" or kill summary).
- **Then:** exit 0; stdout contains output (either "no active processes" or kill summary); v::0 accepted as valid verbosity level (not rejected as unknown)
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-5: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:**
  `clv .ps.kill format::JSON`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr contains error referencing case-sensitive format value or listing valid options; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-6: Source-level AF — `let _ = send_sig` absent

- **Given:** clean environment
- **When:**
  Code inspection via `std::fs::read_to_string`.
  **Expected:** `let _ = send_sigterm` and `let _ = send_sigkill` absent from `commands/process.rs`.
- **Then:** Both `let _` patterns absent.
**Note:** This is an anti-faking check. The signal-error path cannot be triggered through the binary without process injection; source inspection is the only reliable verification.
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-7: `dry::1 format::json` → JSON object output

- **Given:** clean environment
- **When:**
  `clv .ps.kill dry::1 format::json`
  **Expected:** Exit 0; stdout starts with `{`.
- **Then:** exit 0; stdout is valid JSON starting with `{`; contains dry-run process information; stderr is empty
- **Exit:** 0
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-8: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .ps.kill bogus::x`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout contains "bogus" or "unknown parameter" error message; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-9: `dry::2` → exit 1

- **Given:** clean environment
- **When:**
  `clv .ps.kill dry::2`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references out-of-range boolean value "2" for dry::; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-10: `force::2` → exit 1

- **Given:** clean environment
- **When:**
  `clv .ps.kill force::2`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references out-of-range boolean value "2" for force::; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md](../../../../docs/cli/command/ps.md)

---

### IT-11: `pid::1 dry::1` → non-claude PID rejected or preview

- **Given:** clean environment
- **When:**
  `clv .ps.kill pid::1 dry::1`
  **Expected:** Exit 1 (PID 1 is not a claude process); dry mode skips kill but validation still fires.
- **Then:** exit 1; stderr references that PID 1 is not a claude process; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### IT-12: `pid::99999999` → nonexistent PID

- **Given:** clean environment
- **When:**
  `clv .ps.kill pid::99999999`
  **Expected:** Exit 1; PID not in /proc.
- **Then:** exit 1; stderr references PID not found; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### IT-13: `pid::abc` → non-integer

- **Given:** clean environment
- **When:**
  `clv .ps.kill pid::abc`
  **Expected:** Exit 1; non-integer PID value.
- **Then:** exit 1; error references non-integer pid:: value; no kill executed
- **Exit:** 1
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc310_ps_kill_dry_exits_0` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc311_ps_kill_dry_mentions_sigterm` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc312_ps_kill_dry_force_mentions_sigkill` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc313_ps_kill_v0_accepted` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc314_ps_kill_format_uppercase_rejected` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc315_ps_kill_no_let_underscore_on_send_sig` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc316_ps_kill_dry_format_json` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc251_ps_kill_dry_force_dry_wins` | `tests/cli/cross_cutting_test.rs` |
| `tc317_ps_kill_pid_non_claude_exits_1` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc318_ps_kill_pid_nonexistent_exits_1` | `tests/cli/mutation_ps_kill_test.rs` |
| `tc319_ps_kill_pid_non_integer_exits_1` | `tests/cli/mutation_ps_kill_test.rs` |
