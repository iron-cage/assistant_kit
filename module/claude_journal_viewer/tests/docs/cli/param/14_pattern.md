# Parameter :: `pattern`

Edge case tests for the `pattern` parameter. Tests validate the
required-parameter constraint and the literal-substring matching contract —
`pattern` is matched with `str::contains`, case-sensitively, across the six
searched fields; it is not a regex.

**Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)
**Related:** [param_group/04_search.md](../param_group/04_search.md), [type/03_string.md](../type/03_string.md)

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent on `.search` -> exit 1, required parameter missing | Required | ✅ | `viewer_integration_test.rs::ec10_type_validation_at_parse_time` |
| EC-2 | `pattern::"rate limit"` -> matches captured `stdout` | Parsing | ✅ | `viewer_integration_test.rs::ec5_search_pattern_filters_events` |
| EC-3 | `pattern::"(?i)panic"` -> matched literally, **not** case-insensitively | Literal Matching | ⏳ | — |

## Test Coverage Summary

- Required: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Literal Matching: 1 test (EC-3)

**Total:** 3 edge cases

## Architectural Constraint

**There is no regex engine to be case-insensitive with.** The crate has no
`regex` dependency at all — `.search` calls `str::contains`. EC-3 previously
asserted that `(?i)panic` matched both `Panic` and `panic`; it matches neither
unless the output literally contains the eight characters `(?i)pani`. The case
is kept, inverted, because a reader arriving with regex habits needs the
negative result pinned somewhere.

**A regex-shaped pattern never errors.** It is accepted, searched for
literally, and exits 0 finding nothing — which is why the old EC-3's failure
mode was invisible. Nothing in the pipeline is in a position to say "that looks
like a regex and this is not a regex search."

**`message` is the sixth searched field, and was added late.** EC-2's fixture
places the phrase in `stdout` specifically, so a pass proves the output fields
are read rather than proving only that *something* matched. The prompt half is
pinned separately (→ [command/04_search.md](../command/04_search.md) IT-7),
because a fixture that satisfies both at once cannot tell which one carried the
match.

## Test Cases

---

### EC-1: Absent on `.search` -> exit 1, required parameter missing

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr reads `Error: pattern:: parameter required`
- **Exit:** 1
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### EC-2: `pattern::"rate limit"` -> matches captured `stdout`

- **Given:** journal with one event whose `stdout` contains "rate limit" and three that do not
- **When:** `clj .search pattern::"rate limit" since::9999d`
- **Then:** exit 0; the matching event is shown with `(matched)` and the footer reports `1 match`
- **Exit:** 0
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### EC-3: `pattern::"(?i)panic"` -> matched literally, not case-insensitively

- **Given:** journal with three events — one whose `stdout` contains `Panic`, one `panic`, and one the literal text `(?i)panic`
- **When:** `clj .search pattern::"(?i)panic"`
- **Then:** exit 0; **only the literal-text event matches** — the `(?i)` is four characters, not a flag, and case-sensitivity is unaffected
- **And:** `clj .search pattern::"panic"` then matches the lowercase event and the literal one, but not `Panic` — confirming case-sensitivity directly
- **Exit:** 0 both times
- **Note:** the three-event fixture is what makes this non-vacuous. With only `Panic`/`panic` present, a build that *did* implement regex and one that did not would both return some non-empty result, and the case would pass either way
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md), [param_group/04_search.md](../param_group/04_search.md)
