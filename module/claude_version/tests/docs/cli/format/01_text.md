# Format Test: text

### Scope

- **Purpose**: FM- test cases for the `text` output format rendering contract.
- **Responsibility**: Verify labeled output, verbosity interaction, and human-readable structure.
- **In Scope**: Default format selection, label style, verbosity levels v::0 and v::1, non-JSON structure.
- **Out of Scope**: JSON format (→ `02_json.md`), format parameter parsing (→ `../param/05_format.md`).

Format test surface for `text` output. See [cli/format/01_text.md](../../../../docs/cli/format/01_text.md) for specification.

## Test Case Index

| FM | Scenario | Source fn |
|----|----------|-----------|
| FM-1 | Default format (no `format::`) produces labeled `key: value` text output | ✅ |
| FM-2 | `v::0` with text format suppresses labels — raw value only | ✅ |
| FM-3 | `v::1` (default verbosity) produces labeled key: value pairs | ✅ |
| FM-4 | Text output is not valid JSON | ✅ |
| FM-5 | Explicit `format::text` accepted; same labeled output as default | ✅ `fm05_01_text_explicit_format` |

**Total:** 5 tests

---

### FM-1: default format produces labeled text output

- **Given:** no `format::` argument supplied
- **When:** `clv .version.show`
- **Then:** stdout contains `version:` label and a version string; not a JSON object; exit 0

---

### FM-2: v::0 suppresses labels

- **Given:** text format (default), `v::0` supplied
- **When:** `clv .version.show v::0`
- **Then:** stdout contains version string without `version:` label; exit 0

---

### FM-3: v::1 produces labeled output

- **Given:** text format (default), `v::1` supplied
- **When:** `clv .version.show v::1`
- **Then:** stdout contains `version:` label followed by version string; exit 0

---

### FM-4: text output is not parseable as JSON

- **Given:** text format (default)
- **When:** `clv .status`
- **Then:** stdout is not a valid JSON object or array (does not begin with `{` or `[`); exit 0

---

### FM-5: explicit `format::text` accepted; same labeled output as default

- **Given:** clean environment
- **When:** `clv .version.show format::text`
- **Then:** exit 0; stdout contains `version:` label and a version string; output structurally matches invocation with no `format::` argument

---

### Source Functions

| Function | File |
|----------|------|
| `fm01_01_text_default_labeled` | `integration/format_surface_test.rs` |
| `fm02_01_text_v0_raw` | `integration/format_surface_test.rs` |
| `fm03_01_text_v1_labeled` | `integration/format_surface_test.rs` |
| `fm04_01_text_not_json` | `integration/format_surface_test.rs` |
