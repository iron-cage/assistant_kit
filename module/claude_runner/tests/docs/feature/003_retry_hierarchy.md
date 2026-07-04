# Test: Feature — Retry Hierarchy

### Scope

- **Purpose**: FT- test cases verifying the 3-tier retry count and delay resolution contracts for `clr` subprocess retries.
- **Responsibility**: Acceptance criteria confirming per-class Tier 2 budgets, per-invocation Tier 1 override, Tier 3 global fallback, Auth retry parity, and delay tier priority ordering.
- **In Scope**: `--retry-on-transient`, `--retry-on-auth`, `--retry-override`, `--retry-default`, `--retry-override-delay`, `--transient-delay`, `--retry-default-delay`, tier precedence resolution for both count and delay.
- **Out of Scope**: retry/timeout journal event emission (-> `002_journaling_integration.md`), default command assembly (-> `001_runner_tool.md`).

Test case planning for [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md). Tests validate the 3-tier retry count and delay resolution contracts: per-class Tier 2 budgets, per-invocation Tier 1 override, Tier 3 global fallback, Auth retry parity, and delay tier priority.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Tier 2 per-class budget governs retry count | Tier 2 Count Resolution |
| FT-2 | Tier 3 fallback fires for unspecified class | Tier 3 Fallback |
| FT-3 | Auth uses 3-tier retry resolution — same default of 2 as all classes (BUG-325) | Auth Retry Behavior |
| FT-4 | Tier 1 override 0 disables retries for all classes | Tier 1 Override |
| FT-5 | Tier 1 override beats Tier 2 class-specific budget | Tier 1 Override |
| FT-6 | Tier 3 fallback customized via --retry-default | Tier 3 Fallback |
| FT-7 | Tier 2 class-specific beats Tier 3 --retry-default | Tier Priority Ordering |
| FT-8 | Delay tier mirrors count tier — override-delay beats class-delay beats default-delay | Delay Tier Resolution |

## Test Coverage Summary

- Tier 2 Count Resolution: 1 test (FT-1)
- Tier 3 Fallback: 2 tests (FT-2, FT-6)
- Auth Retry Behavior: 1 test (FT-3)
- Tier 1 Override: 2 tests (FT-4, FT-5)
- Tier Priority Ordering: 1 test (FT-7)
- Delay Tier Resolution: 1 test (FT-8)

**Total:** 8 tests

---

### FT-1: Tier 2 per-class budget governs retry count

- **Given:** fake claude binary that exits 2 on first N invocations then exits 0; `--retry-on-transient 5`
- **When:** `clr --retry-on-transient 5 "task"` with fake binary in PATH
- **Then:** `clr` retries up to 5 times on exit 2 with no quota message; succeeds when fake binary eventually exits 0; total invocation count equals failure count + 1 success
- **Exit:** 0
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-001; [cli/param/034_retry_on_transient.md](../../../docs/cli/param/034_retry_on_transient.md)
- **Implemented by:** `retry_transient_test.rs::ec7_transient_retry_succeeds_after_one_failure`, `retry_transient_test.rs::ec8_transient_retry_exhausted_exits_2`

---

### FT-2: Tier 3 fallback fires for unspecified class

- **Given:** fake claude binary that exits 2 (transient) on all invocations; no `--retry-on-transient` or `--retry-override` set
- **When:** `clr "task"` with fake binary in PATH (only --retry-default governs)
- **Then:** `clr` retries exactly 2 times (Tier 3 default of 2) then exits 2 with the subprocess exit code; retry-default fires because neither Tier 1 nor Tier 2 is configured for this class
- **Exit:** 2
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-005; [cli/param/056_retry_default.md](../../../docs/cli/param/056_retry_default.md)
- **Implemented by:** `retry_transient_test.rs::ec10_transient_fallback_default_fires_without_flag`, `retry_default_test.rs::ec8_retry_default_fires_when_no_class_or_override`

---

### FT-3: Auth uses 3-tier retry resolution — same default of 2 as all classes (BUG-325)

- **Given:** fake claude binary that always exits with auth failure; `--retry-on-auth 2 --auth-delay 0`
- **When:** `clr -p --retry-on-auth 2 --auth-delay 0 --max-sessions 0 "task"` with fake binary in PATH
- **Then:** `clr` retries 2 times (budget exhausted); total invocation count = 3 (1 initial + 2 retries); "retries exhausted" in stderr; `[Auth]` class label present; exit 1
- **Exit:** 1
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-002; [cli/param/042_retry_on_auth.md](../../../docs/cli/param/042_retry_on_auth.md)
- **Implemented by:** `retry_auth_test.rs::ec8_auth_error_exhausts_retry_budget`

---

### FT-4: Tier 1 override 0 disables retries for all classes

- **Given:** fake claude binary that always fails (any exit code); `--retry-on-transient 5`, `--retry-default 3`
- **When:** `clr --retry-override 0 --retry-on-transient 5 --retry-default 3 "task"` with fake binary
- **Then:** `clr` performs exactly 1 attempt (no retries) and exits immediately; Tier 1 override of 0 suppresses all class-specific and fallback retry budgets unconditionally
- **Exit:** non-zero (first subprocess failure)
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-003; [cli/param/054_retry_override.md](../../../docs/cli/param/054_retry_override.md)
- **Implemented by:** `retry_override_test.rs::ec7_retry_override_zero_disables_all_retries`

---

### FT-5: Tier 1 override beats Tier 2 class-specific budget

- **Given:** fake claude binary that exits 2 (transient); `--retry-on-transient 0`, `--retry-override 3`
- **When:** `clr --retry-override 3 --retry-on-transient 0 "task"` with fake binary
- **Then:** `clr` retries 3 times (Tier 1 value); the class-specific `--retry-on-transient 0` is ignored; Tier 1 applies to all classes unconditionally
- **Exit:** 2 (budget exhausted)
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-004, AC-006; [cli/param/054_retry_override.md](../../../docs/cli/param/054_retry_override.md)
- **Implemented by:** `retry_override_test.rs::ec8_retry_override_beats_class_specific_zero`, `retry_override_test.rs::ec9_retry_override_applies_to_service_class`

---

### FT-6: Tier 3 fallback customized via --retry-default

- **Given:** fake claude binary that always exits 2; `--retry-default 5`; no class-specific or override params
- **When:** `clr --retry-default 5 "task"` with fake binary
- **Then:** `clr` retries exactly 5 times then exits 2; `--retry-default` sets the Tier 3 global fallback for all classes without higher-tier configuration
- **Exit:** 2
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-007; [cli/param/056_retry_default.md](../../../docs/cli/param/056_retry_default.md)
- **Implemented by:** `retry_default_test.rs::ec8_retry_default_fires_when_no_class_or_override`, `retry_default_test.rs::ec9_retry_default_fires_for_account_class`

---

### FT-7: Tier 2 class-specific beats Tier 3 --retry-default

- **Given:** fake claude binary that always exits 2 (transient); `--retry-on-transient 1`, `--retry-default 5`
- **When:** `clr --retry-on-transient 1 --retry-default 5 "task"` with fake binary
- **Then:** `clr` retries exactly 1 time (Tier 2 class-specific value); `--retry-default 5` is ignored for the transient class because Tier 2 is configured; Tier 2 beats Tier 3 in priority order
- **Exit:** 2
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-007 (converse); [cli/param/034_retry_on_transient.md](../../../docs/cli/param/034_retry_on_transient.md), [cli/param/056_retry_default.md](../../../docs/cli/param/056_retry_default.md)
- **Implemented by:** `retry_default_test.rs::ec7_class_specific_beats_retry_default`

---

### FT-8: Delay tier mirrors count tier — override-delay beats class-delay beats default-delay

- **Given:** fake claude binary that always fails; `--retry-on-transient 1`; `--retry-override-delay` set to 0 (immediate); class-level and default delays are non-zero
- **When:** `clr --retry-on-transient 1 --retry-override-delay 0 "task"` with fake binary and `--retry-override` also set to drive Tier 1 delay path
- **Then:** retry fires immediately (0 s delay) — Tier 1 delay (`--retry-override-delay 0`) takes priority over per-class delay (`--transient-delay`) and global default (`--retry-default-delay`); delay resolution follows the same 3-tier structure as count resolution
- **Exit:** non-zero (budget exhausted)
- **Source:** [feature/003_retry_hierarchy.md](../../../docs/feature/003_retry_hierarchy.md) AC-008; [cli/param/055_retry_override_delay.md](../../../docs/cli/param/055_retry_override_delay.md), [cli/param/035_transient_delay.md](../../../docs/cli/param/035_transient_delay.md), [cli/param/057_retry_default_delay.md](../../../docs/cli/param/057_retry_default_delay.md)
- **Implemented by:** `retry_transient_test.rs::ec7_transient_delay_zero_immediate_retry`
