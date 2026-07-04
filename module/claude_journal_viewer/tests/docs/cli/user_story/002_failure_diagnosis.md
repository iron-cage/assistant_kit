# Test: Failure Diagnosis

### Scope

- **Purpose**: US- acceptance tests verifying developers can quickly find and examine failed CLR invocations.
- **Responsibility**: Acceptance criteria coverage for the failure diagnosis workflow.
- **In Scope**: Exit-code filtering, full-text error search, real-time retry tailing, diagnostic context fields, full-level stdout/stderr inspection.
- **Out of Scope**: Cost analysis (-> `001_cost_tracking.md`), compliance auditing (-> `003_automation_audit.md`).

Test case planning for [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.list exit::2 since::1d` shows all rate-limit failures in last day | Exit Filtering |
| US-2 | `.list exit::4 since::1d` shows all timeout failures in last day | Exit Filtering |
| US-3 | `.search pattern::"rate limit" since::7d` finds rate limit messages | Text Search |
| US-4 | `.search pattern::"error" include_stdout::1` searches subprocess output | Text Search |
| US-5 | `.tail type::retry` follows retry events in real-time | Live Tailing |
| US-6 | Event output includes exit code, error class, duration, and model | Diagnostic Context |
| US-7 | When journal level is `full`, stdout/stderr content is available for inspection | Full-Level Inspection |

## Test Coverage Summary

- Exit Filtering: 2 tests (US-1, US-2)
- Text Search: 2 tests (US-3, US-4)
- Live Tailing: 1 test (US-5)
- Diagnostic Context: 1 test (US-6)
- Full-Level Inspection: 1 test (US-7)

**Total:** 7 tests

---

### US-1: `.list exit::2 since::1d` shows all rate-limit failures in last day

- **Given:** journal with a mix of exit codes including rate-limit failures (exit 2) within the last day
- **When:** `clj .list exit::2 since::1d`
- **Then:** exit 0; only exit-code-2 events from the last day are shown
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-01

---

### US-2: `.list exit::4 since::1d` shows all timeout failures in last day

- **Given:** journal with a mix of exit codes including timeout failures (exit 4) within the last day
- **When:** `clj .list exit::4 since::1d`
- **Then:** exit 0; only exit-code-4 events from the last day are shown
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-02

---

### US-3: `.search pattern::"rate limit" since::7d` finds rate limit messages

- **Given:** journal with at least one event containing "rate limit" in its message within the last 7 days
- **When:** `clj .search pattern::"rate limit" since::7d`
- **Then:** exit 0; matching events are returned
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-03

---

### US-4: `.search pattern::"error" include_stdout::1` searches subprocess output

- **Given:** journal with `full`-level events whose captured stdout contains "error"
- **When:** `clj .search pattern::"error" include_stdout::1`
- **Then:** exit 0; events matched via stdout content are returned, not just message fields
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-04

---

### US-5: `.tail type::retry` follows retry events in real-time

- **Given:** `clj .tail type::retry` running as a background process against a live journal dir
- **When:** a new retry event is appended to the journal
- **Then:** the new retry event appears in tail output within a bounded time; non-retry events are not shown
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-05

---

### US-6: Event output includes exit code, error class, duration, and model

- **Given:** journal with a failed event carrying exit code, error class, duration, and model fields
- **When:** `clj .list` renders that event
- **Then:** exit code, error class, duration, and model are all present in the output
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-06

---

### US-7: When journal level is `full`, stdout/stderr content is available for inspection

- **Given:** journal events recorded at `full` level with captured stdout/stderr
- **When:** `clj .list` or `clj .search` inspects such an event in detail
- **Then:** stdout and stderr content is retrievable for that event
- **Exit:** 0
- **Source:** [user_story/002_failure_diagnosis.md](../../../../docs/cli/user_story/002_failure_diagnosis.md) AC-07
