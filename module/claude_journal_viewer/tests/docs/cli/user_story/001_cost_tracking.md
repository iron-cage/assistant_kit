# Test: Cost Tracking

### Scope

- **Purpose**: US- acceptance tests verifying developers can track and optimize API costs by model, day, and command.
- **Responsibility**: Acceptance criteria coverage for the cost tracking workflow.
- **In Scope**: Cost-sorted listing, cost aggregation by model/day/command, dashboard cost chart, cost/token display format.
- **Out of Scope**: Failure diagnosis (-> `002_failure_diagnosis.md`), capacity/retention (-> `004_capacity_planning.md`).

Test case planning for [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.list sort::cost reverse::1 since::7d` shows most expensive invocations first | Cost Ranking |
| US-2 | `.stats by::model since::30d` shows per-model cost breakdown with totals | Cost Breakdown |
| US-3 | `.stats by::day since::7d` shows daily cost trend | Cost Trend |
| US-4 | `.stats by::command since::30d` shows run vs ask cost split | Cost Breakdown |
| US-5 | `.serve` dashboard displays cost chart over time | Dashboard |
| US-6 | Cost values are displayed in USD with 4 decimal places | Display Format |
| US-7 | Token counts (in/out) are displayed alongside costs | Display Format |

## Test Coverage Summary

- Cost Ranking: 1 test (US-1)
- Cost Breakdown: 2 tests (US-2, US-4)
- Cost Trend: 1 test (US-3)
- Dashboard: 1 test (US-5)
- Display Format: 2 tests (US-6, US-7)

**Total:** 7 tests

---

### US-1: `.list sort::cost reverse::1 since::7d` shows most expensive invocations first

- **Given:** journal with events spanning more than 7 days, with varying cost values
- **When:** `clj .list sort::cost reverse::1 since::7d`
- **Then:** exit 0; only last-7-day events shown, ordered highest cost first
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-01

---

### US-2: `.stats by::model since::30d` shows per-model cost breakdown with totals

- **Given:** journal with events across multiple models within the last 30 days
- **When:** `clj .stats by::model since::30d`
- **Then:** exit 0; output contains one row per model with a cost total per row plus a grand total
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-02

---

### US-3: `.stats by::day since::7d` shows daily cost trend

- **Given:** journal with events spread across the last 7 days
- **When:** `clj .stats by::day since::7d`
- **Then:** exit 0; output contains one row per day showing that day's cost total
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-03

---

### US-4: `.stats by::command since::30d` shows run vs ask cost split

- **Given:** journal with both `run` and `ask` command events within the last 30 days
- **When:** `clj .stats by::command since::30d`
- **Then:** exit 0; output contains separate cost totals for `run` and `ask`
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-04

---

### US-5: `.serve` dashboard displays cost chart over time

- **Given:** journal with events spread across multiple days; `.serve` running
- **When:** dashboard is loaded in a browser
- **Then:** page renders a chart showing cost over time
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-05

---

### US-6: Cost values are displayed in USD with 4 decimal places

- **Given:** journal with events carrying fractional cost values
- **When:** `clj .list` or `clj .stats` renders a cost column
- **Then:** each cost value is formatted as USD with exactly 4 decimal places
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-06

---

### US-7: Token counts (in/out) are displayed alongside costs

- **Given:** journal with events carrying input/output token counts
- **When:** `clj .list` or `clj .stats` renders a cost column
- **Then:** input and output token counts are shown alongside each cost value
- **Exit:** 0
- **Source:** [user_story/001_cost_tracking.md](../../../../docs/cli/user_story/001_cost_tracking.md) AC-07
