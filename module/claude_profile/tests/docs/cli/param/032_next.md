# Test: `next::` Parameter

Edge case coverage for the `next::` parameter on `.usage`. See [param/032_next.md](../../../../docs/cli/param/032_next.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `next::all` accepted with empty credential store | Valid Value |
| EC-2 | `next::session` accepted with empty credential store | Valid Value |
| EC-3 | `next::endurance` accepted with empty credential store | Valid Value |
| EC-4 | `next::drain` accepted with empty credential store | Valid Value |
| EC-5 | `next::reset` accepted with empty credential store | Valid Value |
| EC-6 | `next::bogus` exits 1 naming all five valid values | Invalid Value |
| EC-7 | `next::all` footer omitted when no accounts have valid quota data | Footer Threshold |
| EC-8 | `next::session format::json` — `next::` does not affect JSON output | JSON No-op |

---

### EC-1: `next::all` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::all`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-2: `next::session` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::session`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-3: `next::endurance` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-4: `next::drain` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::drain`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-5: `next::reset` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::reset`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-6: `next::bogus` exits 1 naming all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: "all", "session", "endurance", "drain", "reset".
- **Exit:** 1
- **Source:** [feature/023_next_account_strategies.md AC-07](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-7: `next::all` footer omitted when no accounts have valid quota data

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage next::all`
- **Then:** Exits 0. Stdout does NOT contain "Next by strategy:" (footer is suppressed when fewer than 2 accounts have valid quota data).
- **Exit:** 0
- **Source:** [feature/023_next_account_strategies.md AC-09](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-8: `next::session format::json` — `next::` does not affect JSON output

- **Given:** Two saved accounts with valid credential files (no accessToken — will produce error rows in text, but the JSON array is still emitted).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage next::session format::json`
- **Then-A and Then-B:** Both produce identical JSON arrays (alphabetical, no `"->"` markers). `next::` has no effect on JSON output.
- **Exit:** 0 both cases
- **Source:** [feature/023_next_account_strategies.md AC-08](../../../../docs/feature/023_next_account_strategies.md)
