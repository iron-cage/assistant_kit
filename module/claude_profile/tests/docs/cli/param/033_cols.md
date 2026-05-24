# Test: `cols::` Parameter

Edge case coverage for the `cols::` parameter on `.usage`. See [param/033_cols.md](../../../../docs/cli/param/033_cols.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `cols::+sub` shows Sub column in table header | Show Off-Default Column |
| EC-2 | `cols::-renews` hides Renews column from table header | Hide Default Column |
| EC-3 | `cols::+bogus` exits 1 naming valid column IDs | Invalid Column ID |
| EC-4 | `cols::+sub,-7d_son` adds Sub and removes 7d(Son) simultaneously | Composite Modifier |
| EC-5 | `cols::+7d_son_reset` shows 7d Son Reset (off-by-default column) | Show Off-Default Column |
| EC-6 | `cols::+sub format::json` — `cols::` does not affect JSON output | JSON No-op |
| CC-1 | `flag` and `account` columns always present regardless of `cols::` | Structural Columns Always-On |

---

### EC-1: `cols::+sub` shows Sub column in table header

- **Given:** One saved account with a valid credential file (no accessToken — produces an error row, but the table header is still rendered).
- **When:** `clp .usage cols::+sub`
- **Then:** Exits 0. Table header contains "Sub". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [feature/009_token_usage.md AC-22](../../../../docs/feature/009_token_usage.md)

---

### EC-2: `cols::-renews` hides Renews column from table header

- **Given:** One saved account with a valid credential file (no accessToken).
- **When:** `clp .usage cols::-renews`
- **Then:** Exits 0. Table header does NOT contain "~Renews". Remaining default columns (Expires, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset) are still present.
- **Exit:** 0
- **Source:** [param/033_cols.md](../../../../docs/cli/param/033_cols.md)

---

### EC-3: `cols::+bogus` exits 1 naming valid column IDs

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage cols::+bogus`
- **Then:** Exits 1. Stderr names valid column IDs (e.g., "status", "expires", "sub", "renews", "5h_left", "5h_reset", "7d_left", "7d_son", "7d_reset", "7d_son_reset").
- **Exit:** 1
- **Source:** [feature/009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md)

---

### EC-4: `cols::+sub,-7d_son` adds Sub and removes 7d(Son) simultaneously

- **Given:** One saved account with a valid credential file (no accessToken).
- **When:** `clp .usage cols::+sub,-7d_son`
- **Then:** Exits 0. Table header contains "Sub"; table header does NOT contain "7d(Son)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/033_cols.md](../../../../docs/cli/param/033_cols.md)

---

### EC-5: `cols::+7d_son_reset` shows 7d Son Reset (off-by-default column)

- **Given:** One saved account with a valid credential file (no accessToken).
- **When:** `clp .usage cols::+7d_son_reset`
- **Then:** Exits 0. Table header contains "7d Son Reset". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/033_cols.md](../../../../docs/cli/param/033_cols.md)

---

### EC-6: `cols::+sub format::json` — `cols::` does not affect JSON output

- **Given:** Two saved accounts with valid credential files (no accessToken).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage cols::+sub format::json`
- **Then-A and Then-B:** Both produce identical JSON arrays. No `"sub"` key appears in either output (field-presence params only affect text format; `format::json` always includes all fields through its own schema).
- **Exit:** 0 both cases
- **Source:** [param/033_cols.md](../../../../docs/cli/param/033_cols.md)

---

### CC-1: `flag` and `account` columns always present regardless of `cols::`

- **Behavioral Invariant:** The first two columns — the flag column (`✓`/`→`/`*` markers) and the account name column — are structural and cannot be removed by `cols::` modifiers. They are always rendered.
- **Given:** One saved account with a valid credential file (no accessToken).
- **When:** `clp .usage cols::-renews,-7d_son,-7d_reset`
- **Then:** Exits 0. Stdout still contains the account name (name column present). No error.
- **Exit:** 0
- **Source:** [param/033_cols.md](../../../../docs/cli/param/033_cols.md)
