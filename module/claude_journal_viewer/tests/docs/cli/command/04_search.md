# Test: `.search`

### Scope

- **Purpose**: Verify `.search` performs regex search across event data with correct filtering.
- **Responsibility**: Test case coverage for all 7 `.search` parameters, including the required `pattern`.
- **In Scope**: Regex matching against message field, optional stdout search, combined filters, required-param validation.
- **Out of Scope**: Non-regex listing (-> `01_list.md`), export of matches (-> `08_export.md`).

Test case planning for [command/04_search.md](../../../../docs/cli/command/04_search.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `pattern::"rate limit"` -> finds matching events | Happy Path |
| IT-2 | `pattern::"error" since::1d` -> combined pattern and time filter | Combined Filter |
| IT-3 | `pattern::"timeout" type::timeout` -> combined pattern and type filter | Combined Filter |
| IT-4 | `include_stdout::1` -> searches stdout content too | Stdout Search |
| IT-5 | Missing `pattern` -> exit 1, error message | Required Param |
| IT-6 | No matches -> exit 1 | No Results |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Combined Filter: 2 tests (IT-2, IT-3)
- Stdout Search: 1 test (IT-4)
- Required Param: 1 test (IT-5)
- No Results: 1 test (IT-6)

**Total:** 6 tests

---

### IT-1: `pattern::"rate limit"` -> finds matching events

- **Given:** journal containing events whose message includes "rate limit"
- **When:** `clj .search pattern::"rate limit"`
- **Then:** exit 0; only events whose message matches the pattern are shown with match context highlighted
- **Exit:** 0
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### IT-2: `pattern::"error" since::1d` -> combined pattern and time filter

- **Given:** journal with matching and non-matching events across multiple days
- **When:** `clj .search pattern::"error" since::1d`
- **Then:** exit 0; only matches within the last day are shown
- **Exit:** 0
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/01_since.md](../../../../docs/cli/param/01_since.md)

---

### IT-3: `pattern::"timeout" type::timeout` -> combined pattern and type filter

- **Given:** journal with timeout-type and non-timeout-type events, some matching the pattern
- **When:** `clj .search pattern::"timeout" type::timeout`
- **Then:** exit 0; only timeout-type events matching the pattern are shown
- **Exit:** 0
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/03_type.md](../../../../docs/cli/param/03_type.md)

---

### IT-4: `include_stdout::1` -> searches stdout content too

- **Given:** journal with an event whose stdout (not message) contains the search pattern
- **When:** `clj .search pattern::"Fix bug" include_stdout::1`
- **Then:** exit 0; the event is returned because its stdout field matched
- **Exit:** 0
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)

---

### IT-5: Missing `pattern` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr states that `pattern` is required
- **Exit:** 1
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### IT-6: No matches -> exit 1

- **Given:** journal with no events matching the given pattern
- **When:** `clj .search pattern::"nonexistent_string_xyz"`
- **Then:** exit 1; no events shown
- **Exit:** 1
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md)
