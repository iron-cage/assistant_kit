# Test: `.version.show`

### Scope

- **Purpose**: Integration test cases for the `.version.show` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for version display.
- **In Scope**: Installed binary detection, verbosity levels, output formats, label reverse-lookup (builtin aliases and custom markers).
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`), marker CRUD (→ `17_version_mark.md`).

Integration test planning for the `.version.show` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare semver string only | Minimum output |
| 1 | `<semver>  [labels]` or bare semver when none match | Nominal |
| 2 | Extended detail (same as 1 if no extra data) | Maximum detail |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | `{"version":"X.Y.Z"}` | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: claude binary availability (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| available | Returns installed version | Happy path |
| unavailable | PATH empty or no claude | Failure: exit 2 |

### Factor 4: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 5: Label presence (custom markers and preferred alias)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no markers, no alias match | No labels resolve to installed version | No brackets in output |
| one custom marker matches | One marker value == installed semver | One label in brackets |
| preferred alias matches | settings.json preferred alias resolves to installed version | Builtin label in brackets |
| multiple markers match | Two or more custom markers share the installed semver | Multiple labels in brackets |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-2 | `.version.show v::0` → bare semver string | P | 0 | F1=0, F3=available | [read_version_test.rs] |
| IT-3 | `.version.show v::1` → semver as first token (digit-leading) | P | 0 | F1=1, F3=available | [read_version_test.rs] |
| IT-4 | `.version.show format::json` → `{"version":"..."}` | P | 0 | F2=json, F3=available | [read_version_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.version.show` with no claude in PATH → exit 2 | N | 2 | F3=unavailable | [read_version_test.rs] |
| IT-5 | `.version.show format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-6 | `.version.show v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-7 | `.version.show bogus::x` → exit 1 | N | 1 | F4=present | new |
| IT-8 | Output goes to stdout only; stderr is empty | P | 0 | F3=available | new |
| IT-9 | `v::1` with matching custom marker → label in brackets | P | 0 | F1=1, F5=one_marker, F3=available | [read_version_test.rs] |
| IT-10 | `v::1` with no matching markers → no brackets in output | P | 0 | F1=1, F5=no_match, F3=available | [read_version_test.rs] |
| IT-11 | `format::json` with matching marker → `labels` array present | P | 0 | F2=json, F5=one_marker, F3=available | [read_version_test.rs] |
| IT-12 | `v::0` ignores labels entirely → bare semver, no brackets | P | 0 | F1=0, F5=one_marker, F3=available | [read_version_test.rs] |

### Summary

- **Total:** 12 tests (8 positive, 4 negative)
- **Negative ratio:** 33.3% ⚠️ (below 40% — label tests are all positive by nature; negative coverage already met by IT-1/5/6/7)
- **TC range:** IT-1 to IT-12

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-2, IT-3, IT-4, IT-8, IT-9, IT-10, IT-11, IT-12 |
| 1 | Invalid arguments | IT-5, IT-6, IT-7 |
| 2 | Runtime error (claude not found) | IT-1 |

### Note on Network Conditionality

IT-2, IT-3, IT-4 are environment-conditional: if claude is not installed in the
test environment, the command exits 2 and the assertions are skipped. The tests use
`if out.status.code() == Some(0)` guards.

IT-1 is the inverse: it explicitly removes claude from PATH to force the exit 2 path.

---

## Test Case Details

---

### IT-1: No claude in PATH → exit 2

- **Given:** `PATH=""`, `HOME=<tmp>`.
- **When:**
  `clv .version.show`
  **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-2: `v::0` → bare semver string

- **Given:** claude installed (environment-conditional).
- **When:**
  `clv .version.show v::0`
  **Expected:** Exit 0; stdout is a semver string only (digits and dots).
- **Then:** bare version string.
**Isolation:** Skipped if exit 2 (claude not installed)
- **Exit:** 0

---

### IT-3: `v::1` → semver as first token (digit-leading)

- **Given:** claude installed.
- **When:**
  `clv .version.show v::1`
  **Expected:** Exit 0; first whitespace-delimited token begins with an ASCII digit (semver).
- **Then:** First token is a digit-leading semver string.
**Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-4: `format::json` → `{"version":"..."}`

- **Given:** claude installed.
- **When:**
  `clv .version.show format::json`
  **Expected:** Exit 0; output contains `"version"` JSON key.
- **Then:** JSON with version field.
**Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-5: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `clv .version.show format::xml`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `clv .version.show v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .version.show bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** clean environment with claude binary available
- **When:** `clv .version.show`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/readme.md](../../../../docs/cli/command/readme.md)

---

---

### IT-9: Matching custom marker → label in brackets (v::1)

- **Given:** claude installed; isolated HOME with `version-markers.json` containing `{"name":"team-pin","value":"<installed-version>","description":""}`.
- **When:**
  `clv .version.show v::1`
- **Then:** Exit 0; stdout contains `[team-pin]`.
- **Isolation:** Skipped if exit 2 (claude not installed)
- **Exit:** 0

---

### IT-10: No matching markers → no brackets in output (v::1)

- **Given:** claude installed; isolated HOME with no `version-markers.json` (or empty markers file).
- **When:**
  `clv .version.show v::1`
- **Then:** Exit 0; stdout does NOT contain `[`.
- **Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-11: `format::json` with matching marker → `labels` array present

- **Given:** claude installed; isolated HOME with `version-markers.json` containing `{"name":"team-pin","value":"<installed-version>","description":"pinned"}`.
- **When:**
  `clv .version.show format::json`
- **Then:** Exit 0; output is valid JSON; `"labels"` key is present; first element has `"name":"team-pin"` and `"kind":"custom"`.
- **Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-12: `v::0` ignores labels → bare semver, no brackets

- **Given:** claude installed; isolated HOME with `version-markers.json` containing `{"name":"team-pin","value":"<installed-version>","description":""}`.
- **When:**
  `clv .version.show v::0`
- **Then:** Exit 0; stdout is a semver string only; no `[` character present.
- **Isolation:** Skipped if exit 2
- **Exit:** 0

---

### Source Functions

| Function | File |
|----------|------|
| `tc107_version_show_no_claude_exits_2` | `tests/cli/read_version_test.rs` |
| `tc108_version_show_v0_bare_string` | `tests/cli/read_version_test.rs` |
| `tc109_version_show_v1_labeled` | `tests/cli/read_version_test.rs` |
| `tc111_version_show_format_json` | `tests/cli/read_version_test.rs` |
| `tc509_version_show_no_claude_error` | `tests/cli/error_messages_test.rs` |
| `it09_version_show_v1_custom_marker_label` | `tests/cli/read_version_test.rs` |
| `it10_version_show_v1_no_markers_no_brackets` | `tests/cli/read_version_test.rs` |
| `it11_version_show_json_labels_array` | `tests/cli/read_version_test.rs` |
| `it12_version_show_v0_no_labels` | `tests/cli/read_version_test.rs` |
| `ft006_marker_label_shown_by_version_show` | `tests/cli/read_version_test.rs` |
