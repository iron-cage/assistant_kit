# Format Test: json

### Scope

- **Purpose**: FM- test cases for the `json` output format rendering contract.
- **Responsibility**: Verify JSON structure, case-sensitive rejection, verbosity interaction, and array vs object shape.
- **In Scope**: JSON validity, top-level shape (object vs array), v::0 field stripping, format:: case sensitivity.
- **Out of Scope**: Text format (→ `01_text.md`), format parameter parsing (→ `../param/05_format.md`).

Format test surface for `json` output. See [cli/format/02_json.md](../../../../docs/cli/format/02_json.md) for specification.

## Test Case Index

| FM | Scenario | Source fn |
|----|----------|-----------|
| FM-1 | `format::json` on single-result command produces valid JSON object `{}` | ✅ |
| FM-2 | `format::json` on `.version.list` produces JSON array `[]` | ✅ |
| FM-3 | `format::JSON` (uppercase) is rejected with exit 1 | ✅ |
| FM-4 | `format::json` with `v::0` — primary payload key is always present | ✅ |
| FM-5 | `format::json` output goes to stdout only; stderr is empty | ✅ |

**Total:** 5 tests

---

### FM-1: single-result command produces JSON object

- **Given:** `format::json` supplied
- **When:** `clv .status format::json`
- **Then:** stdout is a valid JSON object (`{...}`); contains `version` key; exit 0

---

### FM-2: list command produces JSON array

- **Given:** `format::json` supplied
- **When:** `clv .version.list format::json`
- **Then:** stdout is a valid JSON array (`[...]`); exit 0

---

### FM-3: uppercase JSON value is rejected

- **Given:** `format::JSON` (uppercase) supplied
- **When:** `clv .status format::JSON`
- **Then:** exit 1; stderr contains error message

---

### FM-4: v::0 with json — primary key always present

- **Given:** `format::json`, `v::0` supplied
- **When:** `clv .status format::json v::0`
- **Then:** stdout is valid JSON; primary payload key (`version`) is present even at v::0; exit 0

---

### FM-5: JSON output goes to stdout only; stderr is empty

- **Given:** clean environment; `format::json` supplied
- **When:** `clv .status format::json`
- **Then:** exit 0; stdout is valid JSON; stderr is empty

---

### Source Functions

| Function | File |
|----------|------|
| `fm01_02_json_object_output` | `tests/cli/format_surface_test.rs` |
| `fm02_02_json_array_output` | `tests/cli/format_surface_test.rs` |
| `fm03_02_json_case_sensitive` | `tests/cli/format_surface_test.rs` |
| `fm04_02_json_v0_primary_key` | `tests/cli/format_surface_test.rs` |
| `fm05_02_json_stdout_only` | `tests/cli/format_surface_test.rs` |
