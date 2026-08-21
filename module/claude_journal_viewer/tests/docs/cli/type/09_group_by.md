# Type :: `GroupBy`

Validation tests for the `GroupBy` enum — the 4 implemented grouping
dimensions (`day`, `model`, `dir`, `agent`), missing-field bucket handling,
and invalid-variant error handling. The 4 planned dimensions (`hour`,
`command`, `error`, `creds`) have no test cases until implemented.

**Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `day` -> grouped by calendar date | Parsing |
| TC-2 | `model` -> grouped by model name | Parsing |
| TC-3 | `dir` -> rows ranked by descending event count | Ranking |
| TC-4 | `agent` -> rows ranked by descending event count | Ranking |
| TC-5 | Field-less events -> visible `(no dir)` / `(no agent)` buckets | Bucketing |
| TC-6 | Invalid variant -> exit 1 listing implemented values | Error Handling |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Ranking: 2 tests (TC-3, TC-4)
- Bucketing: 1 test (TC-5)
- Error Handling: 1 test (TC-6)

**Total:** 6 test cases

> **Implementation note:** TC-3 through TC-6 are implemented as
> `ec21`–`ec24` in `viewer_integration_test.rs` (task 543); `day`/`model`
> grouping is exercised by `ec4_stats_by_model_shows_aggregation` there.
> Matching is exact lowercase — `by::MODEL` takes the invalid-variant path.

## Test Cases

---

### TC-1: `day` -> grouped by calendar date

- **Given:** journal with events spread across multiple days
- **When:** `clj .stats by::day`
- **Then:** exit 0; one row per `YYYY-MM-DD` date, each with count/OK/fail/cost/tokens
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-2: `model` -> grouped by model name

- **Given:** journal with events across multiple models
- **When:** `clj .stats by::model`
- **Then:** exit 0; one row per model with count and cost, ordered by model name
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-3: `dir` -> rows ranked by descending event count

- **Given:** journal with 3 events in `/tmp/alpha`, 2 in `/tmp/beta`, 1 with no `dir` field
- **When:** `clj .stats by::dir since::9999d`
- **Then:** exit 0; `DIR` header; rows appear in order `/tmp/alpha`, `/tmp/beta`, `(no dir)` (descending count); `Total: 6 event(s)` footer
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md); task 543 AC-01

---

### TC-4: `agent` -> rows ranked by descending event count

- **Given:** journal with 3 events for agent `tester@testhost/tmp/alpha/`, 2 for `tester@testhost/tmp/beta/`, 1 with no `agent_id` field
- **When:** `clj .stats by::agent since::9999d`
- **Then:** exit 0; `AGENT` header; rows appear in descending count order with `(no agent)` last
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md); task 543 AC-01

---

### TC-5: Field-less events aggregate under visible buckets

- **Given:** the TC-3/TC-4 fixture (one event lacking both `dir` and `agent_id`)
- **When:** `clj .stats by::dir` and `clj .stats by::agent`
- **Then:** the `(no dir)` / `(no agent)` row is present and carries count 1 — field-less events are never silently dropped
- **Exit:** 0 for both
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md); task 543 AC-02

---

### TC-6: Invalid variant -> exit 1 listing implemented values

- **Given:** clean environment
- **When:** `clj .stats by::bogus`
- **Then:** exit 1; stderr contains `valid: day, model, dir, agent`
- **Exit:** 1
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md); task 543 AC-03
