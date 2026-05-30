# Test: Display Control Parameter Group

Interaction tests for Group 5 (Display Control: `cols::`, `no_color::`, `abs::`, `get::`). See [param_group/005_display_control.md](../../../../docs/cli/param_group/005_display_control.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `cols::` does not affect `format::json` output | JSON No-op |
| CC-2 | Structural columns (`flag`, `account`) always visible regardless of `cols::` | Structural Always-On |
| CC-3 | `get::` bypasses `cols::` — column visibility modifiers do not affect get:: output | Behavioral Divergence |
| CC-4 | `no_color::1` suppresses emoji but does not affect column visibility from `cols::` | Behavioral Divergence |

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

---

### CC-3: `get::` bypasses `cols::` — column visibility does not affect get:: output

- **Given:** One account with live quota; `5h Left = 70%`.
- **When-A:** `clp .usage cols::-5h_left get::5h_left`
- **When-B:** `clp .usage get::5h_left`
- **Then-A and Then-B:** Both output `70%` (or equivalent). Hiding the `5h_left` column via `cols::` does not suppress `get::` extraction — `get::` reads the underlying data, not the rendered column.
- **Exit:** 0 both cases
- **Live:** yes
- **Source fn:** `it238_lim_it_get_bypasses_cols_restriction` (in `tests/cli/usage_test.rs`)
- **Source:** [param_group/005_display_control.md](../../../../docs/cli/param_group/005_display_control.md)

---

### CC-4: `no_color::1` suppresses emoji without affecting `cols::` column visibility

- **Given:** One account; `cols::+sub` adds the Sub column; `no_color::1` strips emoji.
- **When:** `clp .usage cols::+sub no_color::1`
- **Then:** Exits 0. Sub column is present in output (cols:: still applies). Status column shows text label `ok` instead of `🟢` (no_color:: still applies). Both modifiers are independently active.
- **Exit:** 0
- **Source fn:** `it239_cols_sub_and_no_color_independent` (in `tests/cli/usage_test.rs`)
- **Source:** [param_group/005_display_control.md](../../../../docs/cli/param_group/005_display_control.md)
