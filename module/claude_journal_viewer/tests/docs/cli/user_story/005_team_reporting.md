# Test: Team Reporting

### Scope

- **Purpose**: US- acceptance tests verifying team leads can generate usage reports across team members and shared journals.
- **Responsibility**: Acceptance criteria coverage for the team reporting workflow.
- **In Scope**: Per-model/command stats reports, CSV/JSON export for external reporting, report completeness fields, shared/remote journal directory access.
- **Out of Scope**: Individual cost tracking (-> `001_cost_tracking.md`), capacity/retention (-> `004_capacity_planning.md`).

Test case planning for [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.stats by::model since::30d` produces per-model cost report | Reporting |
| US-2 | `.stats by::command since::30d` shows command distribution | Reporting |
| US-3 | `.export format::csv since::30d output::report.csv` exports for spreadsheets | Export |
| US-4 | `.list format::json since::30d limit::0` exports all events as JSON | Export |
| US-5 | Reports include total cost, total tokens, success rate | Report Completeness |
| US-6 | `.list journal_dir::/shared/journal/ since::30d` reads team journal | Shared Journal |

## Test Coverage Summary

- Reporting: 2 tests (US-1, US-2)
- Export: 2 tests (US-3, US-4)
- Report Completeness: 1 test (US-5)
- Shared Journal: 1 test (US-6)

**Total:** 6 tests

---

### US-1: `.stats by::model since::30d` produces per-model cost report

- **Given:** journal with events across multiple models within the last 30 days
- **When:** `clj .stats by::model since::30d`
- **Then:** exit 0; output contains one row per model with a cost total suitable for a report
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-01

---

### US-2: `.stats by::command since::30d` shows command distribution

- **Given:** journal with both `run` and `ask` command events within the last 30 days
- **When:** `clj .stats by::command since::30d`
- **Then:** exit 0; output shows the invocation count/cost distribution across commands
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-02

---

### US-3: `.export format::csv since::30d output::report.csv` exports for spreadsheets

- **Given:** journal with events from the last 30 days
- **When:** `clj .export format::csv since::30d output::report.csv`
- **Then:** exit 0; `report.csv` exists and opens as a valid spreadsheet-compatible CSV
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-03

---

### US-4: `.list format::json since::30d limit::0` exports all events as JSON

- **Given:** journal with more than the default page size of events within the last 30 days
- **When:** `clj .list format::json since::30d limit::0`
- **Then:** exit 0; output is valid JSON containing every matching event with no limit truncation
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-04

---

### US-5: Reports include total cost, total tokens, success rate

- **Given:** journal with a mix of successful and failed events carrying cost and token fields
- **When:** `clj .stats` produces a report
- **Then:** the report includes total cost, total tokens, and success rate figures
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-05

---

### US-6: `.list journal_dir::/shared/journal/ since::30d` reads team journal

- **Given:** a journal directory at `/shared/journal/` containing team events from the last 30 days
- **When:** `clj .list journal_dir::/shared/journal/ since::30d`
- **Then:** exit 0; events are read from the specified shared directory rather than the default location
- **Exit:** 0
- **Source:** [user_story/005_team_reporting.md](../../../../docs/cli/user_story/005_team_reporting.md) AC-06
