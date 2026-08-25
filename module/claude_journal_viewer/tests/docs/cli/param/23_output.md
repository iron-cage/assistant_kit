# Parameter :: `output`

Edge case tests for the `output` parameter — `.export`'s destination file.
Tests validate that it is **required**, that it writes where it is told, and
that the piped form callers reach for lives on `.list` rather than here.

**Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)
**Related:** [param/29_out.md](29_out.md), [param/09_limit.md](09_limit.md)

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> exit 1, `output:: parameter required` | Required Param | ⏳ | — |
| EC-2 | `output::/tmp/events.csv` -> written to that file, not stdout | Parsing | ⏳ | — |
| EC-3 | `clj .list format::jsonl limit::0` is the piped equivalent | Substitute Path | ⏳ | — |

## Test Coverage Summary

- Required Param: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Substitute Path: 1 test (EC-3)

**Total:** 3 edge cases

## Architectural Constraint

**`output` is required — there is no stdout fallback.** EC-1 previously
asserted the opposite: that an absent `output` printed to stdout. It never did.
`export_output` calls `ok_or_else` on the missing key and exits 1 with
`Error: output:: parameter required`, so the documented "default" was a path the
binary has never taken.

**That mattered more than a wrong default usually does**, because the recipe it
implied — `clj .export format::jsonl | jq …` — is the natural thing to reach
for and it exits 1 with no output to pipe. EC-3 exists to pin the form that
*does* work: `.list format::jsonl limit::0` emits the same serialization to
stdout, and `limit::0` is load-bearing, since `.list` alone caps at 50 while
`.export` never did.

**`output` and `out` are different parameters on different commands.**
`.export` takes `output` (required); `.chart` takes `out` (defaulted to
`usage.svg`). Neither accepts the other's spelling — pinned from the `out` side
in [29_out.md](29_out.md) EC-4.

## Test Cases

---

### EC-1: Absent -> exit 1, `output:: parameter required`

- **Given:** journal with events
- **When:** `clj .export format::jsonl` — no `output::`
- **Then:** exit **1**; stderr reads `Error: output:: parameter required`; nothing is written to stdout
- **Exit:** 1
- **Note:** verify by hand with `clj .export format::jsonl; echo "exit=$?"`
- **Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)

---

### EC-2: `output::/tmp/events.csv` -> written to that file, not stdout

- **Given:** journal with one event; a writable target directory
- **When:** `clj .export format::csv output::/tmp/.../events.csv`
- **Then:** exit 0; the file exists and holds the CSV header row `ts,type,command,model,exit_code,cost_usd,duration_ms` followed by one row per event; stdout carries only the confirmation `Exported N event(s) to <path>`
- **Exit:** 0
- **Note:** the confirmation goes to stdout, so "nothing is printed to stdout" is not the right assertion — the right one is that the *serialized events* are not
- **Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)

---

### EC-3: `clj .list format::jsonl limit::0` is the piped equivalent

- **Given:** journal with more than 50 events
- **When:** `clj .export format::jsonl output::<file>`, then `clj .list format::jsonl limit::0`
- **Then:** both emit the same event stream, one to the file and one to stdout — byte-identical line for line
- **And:** dropping `limit::0` truncates the `.list` side at 50 while the exported file keeps every event; the two agree only when the cap is lifted
- **Exit:** 0 both times
- **Note:** the >50-event fixture is what makes this case non-vacuous. On a small journal the two agree whether or not `limit::0` is passed, and the case would pass while proving nothing
- **Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md), [param/09_limit.md](../../../../docs/cli/param/09_limit.md)
