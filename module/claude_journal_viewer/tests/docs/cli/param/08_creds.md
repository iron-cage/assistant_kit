# Parameter :: `creds`

Edge case tests for the `creds` parameter. Tests validate absence
behavior (all credentials) and exclusion of events with no credential field.

**Source:** [param/08_creds.md](../../../../docs/cli/param/08_creds.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> all credentials shown | Default |
| EC-2 | `creds::prod.json` -> only prod.json events | Exact Match |
| EC-3 | Events from `run`/`ask` (no creds field) -> excluded | Field Absence |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Exact Match: 1 test (EC-2)
- Field Absence: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> all credentials shown

- **Given:** journal with `isolated` events using multiple credential files
- **When:** `clj .list`
- **Then:** exit 0; events using any credential are shown
- **Exit:** 0
- **Source:** [param/08_creds.md](../../../../docs/cli/param/08_creds.md)

---

### EC-2: `creds::prod.json` -> only prod.json events

- **Given:** journal with `isolated` events using `prod.json` and `staging.json`
- **When:** `clj .list creds::prod.json`
- **Then:** exit 0; only events with `creds` field exactly equal to `prod.json` are shown
- **Exit:** 0
- **Source:** [param/08_creds.md](../../../../docs/cli/param/08_creds.md)

---

### EC-3: Events from `run`/`ask` (no creds field) -> excluded

- **Given:** journal with `run`/`ask` events (no `creds` field) alongside `isolated` events with a `creds` field
- **When:** `clj .list creds::prod.json`
- **Then:** exit 0; `run`/`ask` events are excluded from the result, since they carry no `creds` field to match against
- **Exit:** 0
- **Source:** [param/08_creds.md](../../../../docs/cli/param/08_creds.md)
