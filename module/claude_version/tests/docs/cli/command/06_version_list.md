# Test: `.version.list`

Integration test planning for the `.version.list` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

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
| TC-116 | Output includes "stable" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-117 | Output includes "latest" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| IT-2 | `v::0` → names only, no descriptions | P | 0 | F1=0 | [read_commands_test.rs] |
| TC-119 | `v::1` → aliases with descriptions | P | 0 | F1=1 | [read_commands_test.rs] |
| IT-3 | Output is deterministic on two calls | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-121 | `format::json` → valid JSON array or object | P | 0 | F2=json | [read_commands_test.rs] |
| TC-122 | Output includes "month" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-123 | `v::1` shows pinned versions in parens `(vX.Y.Z)` | P | 0 | F1=1 | [read_commands_test.rs] |
| TC-124 | `format::json` has `"value"` field | P | 0 | F2=json | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-4 | `bogus::x` → exit 1, unknown parameter | N | 1 | F3=present | new |
| IT-5 | `format::xml` → exit 1, unknown format | N | 1 | F2=xml | new |
| IT-6 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-7 | `format::json` → valid JSON output | P | 0 | F2=json | new |
| IT-8 | Output is stable across repeated invocations | P | 0 | F1=absent | new |

### Summary

- **Total:** 15 tests (12 positive, 3 negative)
- **Negative ratio:** 20.0% command-local; 40.0% combined with cross-cutting TC-242 to TC-244 ✅
- **TC range:** IT-1 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always) | IT-1 through TC-124 |
| 1 | Invalid arguments | IT-4 through IT-6 |
| 2 | Not applicable (compile-time data, no runtime failures) | — |

### Alias Completeness

All three aliases must appear in output: `stable` (TC-116), `latest` (TC-117), `month` (TC-122).
Pinned values for `stable` and `month` must appear in `v::1` output (TC-123).

### JSON Field Requirements

`format::json` must include at minimum: `"name"` or `"alias"`, and `"value"` (pinned semver or null).
TC-121 verifies array structure. TC-124 verifies `"value"` field presence.

---

## Test Case Details

---

### IT-1: `.version.list` exits 0

- **Given:** clean environment
- **When:**
  `cm .version.list`
  **Expected:** Exit 0 with 3 alias lines.
- **Then:** see spec
- **Exit:** 0

---

### IT-2: `v::0` → names only

- **Given:** clean environment
- **When:**
  `cm .version.list v::0`
  **Expected:** Exit 0; output lines contain only alias names without ` — ` description separators.
- **Then:** names only
- **Exit:** 0

---

### IT-3: Deterministic output on two calls

- **Given:** clean environment
- **When:**
  Run `.version.list` twice.
  **Expected:** Both outputs are identical.
- **Then:** outputs equal
- **Exit:** 0

---

### IT-4: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.list bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-5: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.list format::xml`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.list v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `format::json` → valid JSON output

- **Given:** clean environment
- **When:** `cm .version.list format::json`
- **Then:** stdout is valid JSON containing version alias entries
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: Output is stable across repeated invocations

- **Given:** clean environment
- **When:** `cm .version.list` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)
