# Test: `.version.list`

### Scope

- **Purpose**: Integration test cases for the `.version.list` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for alias listing.
- **In Scope**: Alias resolution, verbosity levels, output formats.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.version.list` command. See [001_commands.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, alias names with descriptions | Default behavior |
| 0 | Names only, no descriptions | Minimum output |
| 1 | Names + pinned semver in parens + description | Nominal |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON array of alias objects | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

**Note:** `.version.list` reads only compile-time constants; no external dependencies.
Every invocation exits 0 in a valid environment. No runtime failures exist.

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.version.list` exits 0 | P | 0 | F1=absent, F2=absent | [read_commands_test.rs] |
| IT-9 | Output includes "stable" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| IT-10 | Output includes "latest" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| IT-2 | `v::0` → names only, no descriptions | P | 0 | F1=0 | [read_commands_test.rs] |
| IT-11 | `v::1` → aliases with descriptions | P | 0 | F1=1 | [read_commands_test.rs] |
| IT-3 | Output is deterministic on two calls | P | 0 | F1=absent | [read_commands_test.rs] |
| IT-12 | `format::json` → valid JSON array or object | P | 0 | F2=json | [read_commands_test.rs] |
| IT-13 | Output includes "month" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| IT-14 | `v::1` shows pinned versions in parens `(vX.Y.Z)` | P | 0 | F1=1 | [read_commands_test.rs] |
| IT-15 | `format::json` has `"value"` field | P | 0 | F2=json | [read_commands_test.rs] |
| IT-7 | `format::json` → valid JSON output | P | 0 | F2=json | new |
| IT-8 | Output is stable across repeated invocations | P | 0 | F1=absent | new |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-4 | `bogus::x` → exit 1, unknown parameter | N | 1 | F3=present | new |
| IT-5 | `format::xml` → exit 1, unknown format | N | 1 | F2=xml | new |
| IT-6 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |

### Summary

- **Total:** 15 tests (12 positive, 3 negative)
- **Negative ratio:** 20.0% command-local; 40.0% combined with cross-cutting `tc242_unknown_format_exits_1`, `tc243_uppercase_format_exits_1`, `tc244_empty_format_exits_1` in `read_commands_test.rs` ✅
- **IT range:** IT-1 to IT-15

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always) | IT-1 through IT-3, IT-7 through IT-15 |
| 1 | Invalid arguments | IT-4 through IT-6 |
| 2 | Not applicable (compile-time data, no runtime failures) | — |

### Alias Completeness

All three aliases must appear in output: `stable` (IT-9), `latest` (IT-10), `month` (IT-13).
Pinned values for `stable` and `month` must appear in `v::1` output (IT-14).

### JSON Field Requirements

`format::json` must include at minimum: `"name"` or `"alias"`, and `"value"` (pinned semver or null).
IT-12 verifies array structure. IT-15 verifies `"value"` field presence.

---

## Test Case Details

---

### IT-1: `.version.list` exits 0

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains 3 alias lines (`stable`, `latest`, `month`)
- **Exit:** 0

---

### IT-2: `v::0` → names only

- **Given:** clean environment
- **When:** `clv .version.list v::0`
- **Then:** exit 0; each output line contains only an alias name; no ` — ` description separator present
- **Exit:** 0

---

### IT-3: Deterministic output on two calls

- **Given:** clean environment
- **When:** `clv .version.list` (run twice in succession)
- **Then:** both stdout captures are byte-identical; output order and content do not change between runs
- **Exit:** 0

---

### IT-4: `bogus::x` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list bogus::x`
- **Then:** exit 1; stderr or stdout mentions unknown parameter
- **Exit:** 1

---

### IT-5: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list format::xml`
- **Then:** exit 1; error message references format or valid values
- **Exit:** 1

---

### IT-6: `v::3` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::3`
- **Then:** exit 1; error references out-of-range verbosity value
- **Exit:** 1

---

### IT-7: `format::json` → valid JSON output

- **Given:** clean environment
- **When:** `clv .version.list format::json`
- **Then:** stdout is valid JSON containing version alias entries
- **Exit:** 0
- **Source:** [001_commands.md](../../../../docs/cli/command/readme.md)

---

### IT-8: Output is stable across repeated invocations

- **Given:** clean environment
- **When:** `clv .version.list` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [001_commands.md](../../../../docs/cli/command/readme.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc115_version_list_exits_0` | `integration/read_commands_test.rs` |
| `tc116_version_list_includes_stable` | `integration/read_commands_test.rs` |
| `tc117_version_list_includes_latest` | `integration/read_commands_test.rs` |
| `tc118_version_list_v0_names_only` | `integration/read_commands_test.rs` |
| `tc119_version_list_v1_has_descriptions` | `integration/read_commands_test.rs` |
| `tc120_version_list_is_idempotent` | `integration/read_commands_test.rs` |
| `tc121_version_list_format_json_array` | `integration/read_commands_test.rs` |
| `tc122_version_list_includes_month` | `integration/read_commands_test.rs` |
| `tc123_version_list_v1_shows_pinned_versions` | `integration/read_commands_test.rs` |
| `tc124_version_list_json_has_value_field` | `integration/read_commands_test.rs` |
