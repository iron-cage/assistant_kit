# Test: `.version.list`

Integration test planning for the `.version.list` command. See [commands.md](../../../../../docs/cli/commands.md) for specification.

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
| TC-115 | `.version.list` exits 0 | P | 0 | F1=absent, F2=absent | [read_commands_test.rs] |
| TC-116 | Output includes "stable" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-117 | Output includes "latest" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-118 | `v::0` → names only, no descriptions | P | 0 | F1=0 | [read_commands_test.rs] |
| TC-119 | `v::1` → aliases with descriptions | P | 0 | F1=1 | [read_commands_test.rs] |
| TC-120 | Output is deterministic on two calls | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-121 | `format::json` → valid JSON array or object | P | 0 | F2=json | [read_commands_test.rs] |
| TC-122 | Output includes "month" alias | P | 0 | F1=absent | [read_commands_test.rs] |
| TC-123 | `v::1` shows pinned versions in parens `(vX.Y.Z)` | P | 0 | F1=1 | [read_commands_test.rs] |
| TC-124 | `format::json` has `"value"` field | P | 0 | F2=json | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-459 | `bogus::x` → exit 1, unknown parameter | N | 1 | F3=present | new |
| TC-460 | `format::xml` → exit 1, unknown format | N | 1 | F2=xml | new |
| TC-461 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |

### Summary

- **Total:** 13 tests (10 positive, 3 negative)
- **Negative ratio:** 23.1% command-local; 43.1% combined with cross-cutting TC-242 to TC-244 ✅
- **TC range:** TC-115 to TC-461

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always) | TC-115 through TC-124 |
| 1 | Invalid arguments | TC-459 through TC-461 |
| 2 | Not applicable (compile-time data, no runtime failures) | — |

### Alias Completeness

All three aliases must appear in output: `stable` (TC-116), `latest` (TC-117), `month` (TC-122).
Pinned values for `stable` and `month` must appear in `v::1` output (TC-123).

### JSON Field Requirements

`format::json` must include at minimum: `"name"` or `"alias"`, and `"value"` (pinned semver or null).
TC-121 verifies array structure. TC-124 verifies `"value"` field presence.

---

## Test Case Details

### TC-115: `.version.list` exits 0

**Goal:** Basic invocation always succeeds (compile-time data).
**Setup:** None.
**Command:** `cm .version.list`
**Expected:** Exit 0 with 3 alias lines.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-118: `v::0` → names only

**Goal:** Minimum verbosity suppresses descriptions and pinned versions.
**Setup:** None.
**Command:** `cm .version.list v::0`
**Expected:** Exit 0; output lines contain only alias names without ` — ` description separators.
**Verification:** No line contains description separator (em-dash or colon+space pattern).
**Pass Criteria:** Exit 0; names only.

---

### TC-120: Deterministic output on two calls

**Goal:** Compile-time data is deterministic and idempotent.
**Setup:** None.
**Command:** Run `.version.list` twice.
**Expected:** Both outputs are identical.
**Verification:** `stdout(out1) == stdout(out2)`.
**Pass Criteria:** Exit 0; outputs equal.

---

### TC-459: `bogus::x` → exit 1

**Goal:** Unknown parameters rejected before reading aliases.
**Setup:** None.
**Command:** `cm .version.list bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-460: `format::xml` → exit 1

**Goal:** Unrecognized format value rejected.
**Setup:** None.
**Command:** `cm .version.list format::xml`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-461: `v::3` → exit 1

**Goal:** Out-of-range verbosity rejected.
**Setup:** None.
**Command:** `cm .version.list v::3`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.
