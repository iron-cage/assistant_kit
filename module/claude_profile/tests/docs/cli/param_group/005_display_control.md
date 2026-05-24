# Test: Display Control Parameter Group

Interaction tests for Group 5 (Display Control: `cols::`). See [param_group/005_display_control.md](../../../../docs/cli/param_group/005_display_control.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `cols::` does not affect `format::json` output | JSON No-op |
| CC-2 | Structural columns (`flag`, `account`) always visible regardless of `cols::` | Structural Always-On |

---

### CC-1: `cols::` does not affect `format::json` output

- **Behavioral Divergence:** `cols::+sub format::json` vs `format::json` — JSON arrays are identical in both cases. Column visibility modifiers only affect text-format table rendering.
- **Given:** Two saved accounts: `a@x.com` and `b@x.com`; both with credential files missing `accessToken` (will produce error rows in JSON but JSON array is still emitted).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage cols::+sub format::json`
- **Then-A and Then-B:** Both produce identical JSON arrays. No `"sub"` key appears in either output (field-presence params only affect text format; `format::json` always includes all fields through its own schema).
- **Exit:** 0 both cases
- **Source:** [feature/009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md), [param/033_cols.md EC-6](../../../../tests/docs/cli/param/033_cols.md)

---

### CC-2: Structural columns (`flag`, `account`) always visible regardless of `cols::`

- **Behavioral Divergence:** Even when multiple optional columns are hidden via `cols::-*`, the flag and account columns are always rendered.
- **Given:** One saved account with a credential file missing `accessToken`.
- **When:** `clp .usage cols::-expires,-renews,-7d_son,-7d_reset`
- **Then:** Exits 0. Stdout still contains the account name (account column present). No error.
- **Exit:** 0
- **Source:** [feature/009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md), [param/033_cols.md CC-1](../../../../tests/docs/cli/param/033_cols.md)
