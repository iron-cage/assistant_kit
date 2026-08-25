# Parameter Group :: Search

Interaction tests for the Search group, which has one member: `pattern`. Tests
validate the required-parameter rule, the substring (not regex) matching
contract, and the interaction between the search scope and the journal level
that decides whether the searched fields hold anything at all. Only used by
`.search`.

**Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)
**Related:** [param/28_include_stdout.md](../param/28_include_stdout.md)

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| CC-1 | `pattern` omitted -> exit 1, required param error | Required Param | ✅ | `viewer_integration_test.rs::ec10_type_validation_at_parse_time` |
| CC-2 | `pattern` matches `stdout` with no flag to enable it | Search Scope | ✅ | `viewer_integration_test.rs::ec5_search_pattern_filters_events` |
| CC-3 | `pattern` is a substring, not a regex | Matching Contract | ⏳ | — |
| CC-4 | At `meta` journal level there is no output to search | Journal Level Interaction | ⏳ | — |

## Test Coverage Summary

- Required Param: 1 test (CC-1)
- Search Scope: 1 test (CC-2)
- Matching Contract: 1 test (CC-3)
- Journal Level Interaction: 1 test (CC-4)

**Total:** 4 corner cases

## Architectural Constraint

**A group of one still has interactions — with the journal, not with a sibling.**
CC-2 through CC-4 previously described a two-member group whose second member
(`include_stdout`) toggled the search scope. It never existed, so the cases
described a toggle between two behaviors when only one was ever implemented. The
cases are kept, retargeted at the interactions `pattern` genuinely has: with the
fields the reader is searched against, and with the journal level that decides
whether those fields carry anything.

**CC-4 is the case that survived intact in substance.** Its claim — at `meta`
level there is no `stdout`/`stderr` stored, so nothing in them can match — never
depended on the flag. What changed is the contrast it draws: not
`include_stdout::1` versus `::0`, but the same command against a `full` journal
versus a `meta` one.

**The level does not gate every searched field.** `message` is written at every
level, so of the six searched fields it is the only one guaranteed present in a
`meta` journal alongside `error_message`, `model`, and `command`. That is why
CC-4 now runs a prompt probe as well as an output probe: without it, "the level
stores less" and "the command is broken" produce the same empty answer.

## Test Cases

---

### CC-1: `pattern` omitted -> exit 1, required param error

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr states `pattern:: parameter required`
- **Exit:** 1
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)

---

### CC-2: `pattern` matches `stdout` with no flag to enable it

- **Given:** journal with an event whose `stdout` contains the pattern and whose other fields do not
- **When:** `clj .search pattern::"rate limit"`
- **Then:** exit 0; the event is returned — `message`, `stdout`, `stderr`, `error_message`, `model`, and `command` are all searched unconditionally
- **Exit:** 0
- **Note:** the group is one member wide precisely because this is unconditional. There is nothing left for a second parameter to widen
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md)

---

### CC-3: `pattern` is a substring, not a regex

- **Given:** journal with an event whose `stdout` is `exit_code: 3` and another whose `stdout` is the literal text `exit_code: [1-9]`
- **When:** `clj .search pattern::"exit_code: [1-9]"`
- **Then:** exit 0; the **literal** event matches and the `exit_code: 3` event does not — the brackets are characters, not a character class
- **Exit:** 0
- **Note:** this is the case that would have caught the regex recipes the user stories carried. A regex-shaped pattern does not error; it quietly matches nothing and exits 0, which reads as a clean audit
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md), [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### CC-4: At `meta` journal level there is no output to search

- **Given:** two journals holding the same commands — one recorded at `full` level, one at `meta` (which stores no `stdout`/`stderr`) — where the needle appears in the subprocess *output*
- **When:** the same `clj .search pattern::"needle"` is run against each
- **Then:** the `full` journal returns the matching event; the `meta` journal exits 0 with `No events matching` — same query, same commands, different recorded fields
- **And:** a phrase from the *prompt* matches in both journals — `message` is written outside the level gate (`cli/execution.rs`: `fields.message` is assigned unconditionally, `fields.stdout`/`stderr` only inside `is_full_level`)
- **Exit:** 0 in every case
- **Note:** the pairing is the assertion, and the prompt half is what keeps it honest. Running only the output probe against the `meta` journal would pass against a `.search` that was broken outright, since both produce no matches; the prompt probe distinguishes "this level stores less" from "this command found nothing"
- **Note:** `clj .status` reports the level in force (`Journal level: full (CLR_JOURNAL=full)`), which is the check to run first when a search that should match returns nothing
- **Source:** [param_group/04_search.md](../../../../docs/cli/param_group/04_search.md), [command/07_status.md](../../../../docs/cli/command/07_status.md)
