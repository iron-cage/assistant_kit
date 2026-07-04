# Parameter Group :: Filtering

Interaction tests for the Filtering group: `since`, `until`, `type`, `command`,
`exit`, `model`, `dir`, `creds`. Tests validate time-window construction and
AND-combination semantics across filter parameters.

**Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `since::1h until::5m` -> window is `[now-1h, now-5m]` | Time Window |
| CC-2 | `since` only -> window is `[now-since, now]` | Time Window |
| CC-3 | `until` only -> window is `[beginning, now-until]` | Time Window |
| CC-4 | `since::5m until::1h` -> inverted window rejected as invalid | Boundary |
| CC-5 | `type::execution command::ask exit::0` -> all AND-combined | Combined |
| CC-6 | `model` filter excludes events missing the model field | Field Absence |

## Test Coverage Summary

- Time Window: 3 tests (CC-1, CC-2, CC-3)
- Boundary: 1 test (CC-4)
- Combined: 1 test (CC-5)
- Field Absence: 1 test (CC-6)

**Total:** 6 corner cases

## Test Cases
---

### CC-1: `since::1h until::5m` -> window is `[now-1h, now-5m]`

- **Given:** journal with events both inside and outside the last hour, and within/outside the last 5 minutes
- **When:** `clj .list since::1h until::5m`
- **Then:** only events between 1 hour ago and 5 minutes ago are shown
- **Exit:** 0
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
---

### CC-2: `since` only -> window is `[now-since, now]`

- **Given:** journal with events both inside and outside the last hour
- **When:** `clj .list since::1h`
- **Then:** only events from the last hour up to now are shown
- **Exit:** 0
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
---

### CC-3: `until` only -> window is `[beginning, now-until]`

- **Given:** journal with events both inside and outside the last 5 minutes
- **When:** `clj .list until::5m`
- **Then:** only events older than 5 minutes ago are shown
- **Exit:** 0
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
---

### CC-4: `since::5m until::1h` -> inverted window rejected as invalid

- **Given:** clean environment
- **When:** `clj .list since::5m until::1h` (since more recent than until, from now)
- **Then:** exit 1; stderr states the since/until combination produces an invalid (empty) time window
- **Exit:** 1
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
---

### CC-5: `type::execution command::ask exit::0` -> all AND-combined

- **Given:** journal with events of varying type, command, and exit code
- **When:** `clj .list type::execution command::ask exit::0`
- **Then:** only events matching all three filters simultaneously are shown
- **Exit:** 0
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
---

### CC-6: `model` filter excludes events missing the model field

- **Given:** journal with some events carrying a `model` field and others without one
- **When:** `clj .list model::opus`
- **Then:** events without a `model` field are excluded from the result, even though the filter is a substring match
- **Exit:** 0
- **Source:** [param_group/01_filtering.md](../../../../docs/cli/param_group/01_filtering.md)
