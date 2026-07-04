# CLI User Stories

### Scope

- **Purpose**: User story index covering persona goals and acceptance criteria.
- **Responsibility**: Define primary user workflows and their acceptance criteria.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_cost_tracking.md` | Track API costs over time by model and project |
| `002_failure_diagnosis.md` | Diagnose failures and identify error patterns |
| `003_automation_audit.md` | Audit automated CLR sessions for compliance |
| `004_capacity_planning.md` | Plan API capacity and manage journal storage |
| `005_team_reporting.md` | Generate usage reports for teams and leads |

### All User Stories (5 total)

| # | User Story | Persona | Primary Commands |
|---|------------|---------|------------------|
| 1 | [Cost Tracking](001_cost_tracking.md) | Developer | .list, .stats, .serve |
| 2 | [Failure Diagnosis](002_failure_diagnosis.md) | Developer | .list, .search, .tail |
| 3 | [Automation Audit](003_automation_audit.md) | Developer | .list, .search, .export |
| 4 | [Capacity Planning](004_capacity_planning.md) | Developer | .stats, .status, .prune |
| 5 | [Team Reporting](005_team_reporting.md) | Lead | .stats, .export |

**Total:** 5 user stories
