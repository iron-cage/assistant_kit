# Parameter Group :: Search

Interaction tests for the Search group: `pattern`, `include_stdout`. Tests
validate the required-parameter rule and the message-vs-stdout/stderr search
scope toggle. Only used by `.search`.

**Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `pattern` omitted -> exit 1, required param error | Required Param |
| CC-2 | `include_stdout::0` (default) -> matches only the `message` field | Search Scope |
| CC-3 | `include_stdout::1` -> also matches `stdout`/`stderr` fields | Search Scope |
| CC-4 | `include_stdout::1` on a `meta`-level event -> no extra matches beyond `message` | Journal Level Interaction |

## Test Coverage Summary

- Required Param: 1 test (CC-1)
- Search Scope: 2 tests (CC-2, CC-3)
- Journal Level Interaction: 1 test (CC-4)

**Total:** 4 corner cases

## Test Cases
---

### CC-1: `pattern` omitted -> exit 1, required param error

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr states `pattern` is required
- **Exit:** 1
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)
---

### CC-2: `include_stdout::0` (default) -> matches only the `message` field

- **Given:** journal with an event whose `stdout` field contains the search pattern but whose `message` field does not
- **When:** `clj .search pattern::"needle"`
- **Then:** the event is NOT returned, since `include_stdout` defaults to 0 and only `message` is searched
- **Exit:** 1 (no matches)
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)
---

### CC-3: `include_stdout::1` -> also matches `stdout`/`stderr` fields

- **Given:** journal with an event whose `stdout` field contains the search pattern but whose `message` field does not
- **When:** `clj .search pattern::"needle" include_stdout::1`
- **Then:** the event IS returned, since `stdout` is now included in the search scope
- **Exit:** 0
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)
---

### CC-4: `include_stdout::1` on a `meta`-level event -> no extra matches beyond `message`

- **Given:** journal recorded at `meta` journal level (no stdout/stderr content stored)
- **When:** `clj .search pattern::"needle" include_stdout::1`
- **Then:** matches are identical to `include_stdout::0` for these events, since there is no stdout/stderr content to search
- **Exit:** 0 or 1 depending on `message` field content
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)
