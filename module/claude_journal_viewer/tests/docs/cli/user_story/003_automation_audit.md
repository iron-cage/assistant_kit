# Test: Automation Audit

### Scope

- **Purpose**: US- acceptance tests verifying developers can audit automated CLR pipeline invocations for compliance and correctness.
- **Responsibility**: Acceptance criteria coverage for the automation audit workflow.
- **In Scope**: Command/directory/credential filtering, CSV export for audit trails, anomaly search, dashboard visual inspection, forensic field completeness.
- **Out of Scope**: Cost analysis (-> `001_cost_tracking.md`), failure diagnosis (-> `002_failure_diagnosis.md`).

Test case planning for [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.list command::ask since::7d` shows all automated ask invocations | Command Filtering |
| US-2 | `.list dir::/ci/project since::1d` filters by project directory | Directory Filtering |
| US-3 | `.list creds::automation.json` shows all runs with specific credentials | Credential Filtering |
| US-4 | `.export format::csv since::7d output::/tmp/audit.csv` produces audit trail | Export |
| US-5 | `.search pattern::"unexpected" include_stdout::1 since::7d` finds anomalies | Anomaly Search |
| US-6 | `.serve` dashboard allows visual inspection of automation patterns | Dashboard |
| US-7 | Export includes all event fields for forensic analysis | Export |

## Test Coverage Summary

- Command Filtering: 1 test (US-1)
- Directory Filtering: 1 test (US-2)
- Credential Filtering: 1 test (US-3)
- Export: 2 tests (US-4, US-7)
- Anomaly Search: 1 test (US-5)
- Dashboard: 1 test (US-6)

**Total:** 7 tests

---

### US-1: `.list command::ask since::7d` shows all automated ask invocations

- **Given:** journal with a mix of `run` and `ask` command events within the last 7 days
- **When:** `clj .list command::ask since::7d`
- **Then:** exit 0; only `ask` command events from the last 7 days are shown
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-01

---

### US-2: `.list dir::/ci/project since::1d` filters by project directory

- **Given:** journal with events from multiple project directories, including `/ci/project`, within the last day
- **When:** `clj .list dir::/ci/project since::1d`
- **Then:** exit 0; only events whose directory matches `/ci/project` are shown
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-02

---

### US-3: `.list creds::automation.json` shows all runs with specific credentials

- **Given:** journal with events recorded under multiple credential files, including `automation.json`
- **When:** `clj .list creds::automation.json`
- **Then:** exit 0; only events using `automation.json` credentials are shown
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-03

---

### US-4: `.export format::csv since::7d output::/tmp/audit.csv` produces audit trail

- **Given:** journal with events from the last 7 days
- **When:** `clj .export format::csv since::7d output::/tmp/audit.csv`
- **Then:** exit 0; `/tmp/audit.csv` exists and contains a header row plus one row per matching event
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-04

---

### US-5: `.search pattern::"unexpected" include_stdout::1 since::7d` finds anomalies

- **Given:** journal with `full`-level events, at least one with "unexpected" in its captured stdout, within the last 7 days
- **When:** `clj .search pattern::"unexpected" include_stdout::1 since::7d`
- **Then:** exit 0; matching anomalous events are returned
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-05

---

### US-6: `.serve` dashboard allows visual inspection of automation patterns

- **Given:** journal with a mix of automated invocations; `.serve` running
- **When:** dashboard is loaded in a browser
- **Then:** page renders a view allowing visual inspection of invocation patterns over time
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-06

---

### US-7: Export includes all event fields for forensic analysis

- **Given:** journal events carrying the full event field set (timestamps, exit code, cost, tokens, credentials, directory, model)
- **When:** `clj .export format::csv` exports those events
- **Then:** every event field is present as a column in the exported file
- **Exit:** 0
- **Source:** [user_story/003_automation_audit.md](../../../../docs/cli/user_story/003_automation_audit.md) AC-07
