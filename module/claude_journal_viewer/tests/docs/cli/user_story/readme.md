# User Story Tests

### Scope

- **Purpose**: Test case planning for user story doc instances in `docs/cli/user_story/`.
- **Responsibility**: Index of per-user-story test case spec files covering end-to-end persona workflows.
- **In Scope**: All 5 `docs/cli/user_story/` doc instances: Cost Tracking, Failure Diagnosis, Automation Audit, Capacity Planning, Team Reporting.
- **Out of Scope**: Command-level tests (-> `../command/`), parameter edge cases (-> `../param/`).

Per-user-story test case indices for `claude_journal_viewer`. See [user_story/readme.md](../../../../docs/cli/user_story/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `001_cost_tracking.md` | US- acceptance tests for cost tracking by model/day/command | ✅ |
| `002_failure_diagnosis.md` | US- acceptance tests for exit-code filtering and error search | ✅ |
| `003_automation_audit.md` | US- acceptance tests for pipeline audit and forensic export | ✅ |
| `004_capacity_planning.md` | US- acceptance tests for journal health and retention | ✅ |
| `005_team_reporting.md` | US- acceptance tests for team-wide reporting and shared journals | ✅ |
| `procedure.md` | Workflow for creating and updating user story test specs | ✅ |
