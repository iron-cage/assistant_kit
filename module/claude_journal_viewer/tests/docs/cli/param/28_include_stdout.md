# Parameter :: `include_stdout`

**Superseded — not a parameter.** Tests validate that `.search` reads `stdout`
and `stderr` with no flag at all, and that the retracted flag is rejected rather
than quietly tolerated.

**Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)
**Related:** [invariant/003_cli_surface_consistency.md](../../invariant/003_cli_surface_consistency.md)

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> `stdout` and `stderr` are searched anyway | Default | ✅ | `viewer_integration_test.rs::ec5_search_pattern_filters_events` |
| EC-2 | `include_stdout::1` -> exit 1, `unknown parameter` | Retraction | ✅ | `viewer_integration_test.rs::ec28_unknown_param_exits_1` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Retraction: 1 test (EC-2)

**Total:** 2 edge cases

## Architectural Constraint

Both cases previously described the opposite behavior — EC-1 asserted that an
absent flag restricted the search to a `message` field, EC-2 that
`include_stdout::1` widened it. Neither was ever implemented, so neither ever
failed. That is the specific hazard a plan-only test case carries: it reads as
coverage in the index, and a reviewer counting rows sees two green-adjacent
entries for a parameter the binary rejects outright.

The old EC-1 was wrong in an ironic second way: it named `message` as the field
the search was restricted to, when `message` was in fact the only text field
`.search` did **not** read. That inversion has since been closed from the other
side — `message` was added to the match set — so all six fields the flag ever
gestured at are now read together, unconditionally, and the flag has even less
left to mean than when it was retracted.

The two cases are kept rather than deleted because the *behavior* they were
reaching for is real and worth pinning — it just belongs to `.search` itself
now, not to a flag. Deleting them would leave `.search`'s unconditional output
search asserted nowhere, which is how the flag came to be documented in the
first place.

## Test Cases

---

### EC-1: Absent -> `stdout` and `stderr` are searched anyway

- **Given:** journal with an event whose `stdout` field contains the pattern and whose `message` field does not
- **When:** `clj .search pattern::"Fix bug"` — no flag
- **Then:** exit 0; the event **is** matched, because `.search` reads `message`, `stdout`, `stderr`, `error_message`, `model`, and `command` unconditionally
- **And:** the fixture keeps the phrase out of `message` on purpose — with the prompt now searched too, a needle present in both fields would no longer prove `stdout` was the one read
- **Exit:** 0
- **Note:** this is the assertion that makes the flag unnecessary. If it ever fails, the flag becomes meaningful again and the tombstone stops being correct
- **Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md); [command/04_search.md](../../../../docs/cli/command/04_search.md)

---

### EC-2: `include_stdout::1` -> exit 1, `unknown parameter`

- **Given:** any journal dir
- **When:** `clj .search pattern::x include_stdout::1`
- **Then:** exit 1 with `unknown parameter` — a retracted flag is an unknown one, with no separate diagnostic class, and the message lists `.search`'s accepted set
- **Exit:** 1
- **Note:** covered alongside `.list wide::`/`columns::` and `.tail since::`/`limit::` in the same case, because all four are the same claim: a parameter removed from the docs must be removed from the binary too, and the binary is where that gets proven
- **Note:** [`tests/cli_doc_consistency.rs`](../../../cli_doc_consistency.rs) does not cover this. DC-2 exempts tombstones by design and DC-4 walks only live pages, so the doc-consistency gates are silent on a retracted parameter's runtime behavior — deliberately, since they read the binary's accepted set as their own input
- **Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)
