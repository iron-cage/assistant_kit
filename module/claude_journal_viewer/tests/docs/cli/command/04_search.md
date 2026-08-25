# Test: `.search`

### Scope

- **Purpose**: Verify `.search` performs substring search across the six searched event fields with correct filtering.
- **Responsibility**: Test case coverage for all 12 `.search` parameters — `pattern` (required), the nine filters it shares with `.list`, and the two global parameters.
- **In Scope**: Substring matching against `message`, `stdout`, `stderr`, `error_message`, `model`, and `command`; the fields deliberately *not* searched; combined filters; required-param validation; the empty result set.
- **Out of Scope**: Non-matching listing (-> `01_list.md`), export of matches (-> `08_export.md`), which parameters `.search` accepts at all (-> `../../invariant/003_cli_surface_consistency.md`).

Test case planning for [command/04_search.md](../../../../docs/cli/command/04_search.md).

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| IT-1 | `pattern::"rate limit"` -> finds matching events | Happy Path | ✅ | `viewer_integration_test.rs::ec5_search_pattern_filters_events` |
| IT-2 | `pattern::"error" since::1d` -> combined pattern and time filter | Combined Filter | ⏳ | — |
| IT-3 | `pattern::"timeout" type::timeout` -> combined pattern and type filter | Combined Filter | ⏳ | — |
| IT-4 | Output fields are searched with no flag; `include_stdout::1` exits 1 | Retraction | ✅ | `viewer_integration_test.rs::ec5_search_pattern_filters_events`, `::ec28_unknown_param_exits_1` |
| IT-5 | Missing `pattern` -> exit 1, error message | Required Param | ✅ | `viewer_integration_test.rs::ec10_type_validation_at_parse_time` |
| IT-6 | No matches -> exit **0** with a stated non-result | No Results | ⏳ | — |
| IT-7 | `message` is searched and `dir` is not — the set is exactly six | Search Scope | ✅ | `viewer_integration_test.rs::ec38_search_reads_prompt_and_only_documented_fields` |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Combined Filter: 2 tests (IT-2, IT-3)
- Retraction: 1 test (IT-4)
- Required Param: 1 test (IT-5)
- No Results: 1 test (IT-6)
- Search Scope: 1 test (IT-7)

**Total:** 7 tests

## Architectural Constraint

**`pattern` is `str::contains`, not a regex.** Three of these cases described
regex matching, and the surrounding user stories offered `pattern::"exit_code:
[1-9]"` as a working recipe. It is not one — the brackets are literal, so the
recipe matched nothing and returned exit 0, which reads as "no anomalies found"
rather than as "this query cannot work."

**Six fields are searched; `message` was the sixth, added late.** `stdout`,
`stderr`, `error_message`, `model`, and `command` were read from the start;
`message` — the prompt the event was launched with — was not, so searching for
a phrase you remember typing returned nothing while the same phrase quoted back
in the model's `stdout` matched. The omission was invisible from the output: a
skipped field cannot narrow a result set in any way a caller can see, so the
answer was exit 0 and `No events matching`, which is what `.search` also says
for a phrase genuinely absent. IT-7 pins the field, in both directions.

**IT-7 is what makes the boundary testable at all.** Its fixture puts the *same*
phrase in `message` on one event and in `dir` — filterable, deliberately not
searched — on another, so one `1 match` assertion fails at 0 if `message` stops
being read and at 2 if the set silently widens. A fixture with the phrase in one
place only could catch the first and never the second.

**The old IT-4 asserted a parameter that was never accepted.** It has been
turned around to assert the two halves of the retraction instead: the output
fields are searched with no flag at all, and `include_stdout::1` exits 1. See
[param/28_include_stdout.md](../param/28_include_stdout.md).

---

### IT-1: `pattern::"rate limit"` -> finds matching events

- **Given:** journal containing an event whose `stdout` includes "rate limit", and three that do not
- **When:** `clj .search pattern::"rate limit" since::9999d`
- **Then:** exit 0; the matching event is shown with a `(matched)` marker, and the footer reports `1 match`
- **Exit:** 0
- **Note:** the fixture places the phrase in `stdout` and nowhere else, so a pass proves `stdout` is searched rather than proving only that *some* field matched
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### IT-2: `pattern::"error" since::1d` -> combined pattern and time filter

- **Given:** journal with matching and non-matching events across multiple days
- **When:** `clj .search pattern::"error" since::1d`
- **Then:** exit 0; only matches within the last day are shown — `since` narrows the candidate set before `pattern` runs
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

### IT-4: Output fields are searched with no flag; `include_stdout::1` exits 1

- **Given:** journal with an event whose `stdout` contains the pattern
- **When:** `clj .search pattern::"rate limit"` with no flag, then `clj .search pattern::x include_stdout::1`
- **Then:** the first exits 0 and returns the event; the second exits **1** with `unknown parameter`, because the flag has been superseded rather than defaulted
- **Exit:** 0, then 1
- **Note:** both halves are needed. Asserting only the rejection would leave the behavior that made the flag redundant unpinned, which is how it came to be documented as a parameter in the first place
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)

---

### IT-5: Missing `pattern` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr states `pattern:: parameter required`
- **Exit:** 1
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### IT-6: No matches -> exit **0** with a stated non-result

- **Given:** journal with no events matching the given pattern
- **When:** `clj .search pattern::"nonexistent_string_xyz"`
- **Then:** exit **0**, printing `No events matching 'nonexistent_string_xyz'.` — "found nothing" is an answer, not a failure
- **Exit:** 0
- **Note:** this case previously specified exit 1, which no build has ever produced. Verify with `clj .search pattern::zzz; echo "exit=$?"`
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### IT-7: `message` is searched and `dir` is not — the set is exactly six

- **Given:** journal with two events carrying the same phrase `refactor the parser` in different fields — one in `message` (with `command::ask`), one in `dir` (with `command::run`); neither has it in `stdout`
- **When:** `clj .search pattern::"refactor the parser"`, then `clj .search pattern::"/w/refactor"`
- **Then:** the first exits 0 reporting `1 match(es)`, and the row shown is the `ask` event; the second exits 0 with `No events matching`
- **Exit:** 0 both times
- **Note:** the shared phrase is the point. `1 match` is a two-sided assertion — `0` means `message` went unread, `2` means `dir` leaked into the searched set — and the `ask` check makes sure the surviving row is the right one of the two rather than merely one of them. The second probe restates the negative alone so a regression names its own direction
- **Note:** mutation-checked. Dropping `message` from the match set produced `No events matching 'refactor the parser'` (log `-0083`); adding `dir` produced `2 match(es)` (log `-0084`)
- **Implemented as:** `ec38_search_reads_prompt_and_only_documented_fields`
- **Source:** [command/04_search.md](../../../../docs/cli/command/04_search.md), [param/07_dir.md](../../../../docs/cli/param/07_dir.md)
