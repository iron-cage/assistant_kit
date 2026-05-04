# Test: `.version.history`

Integration test planning for the `.version.history` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `count::` (Integer, optional, default 10)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 10 | Default behavior |
| 0 | Empty output, no entries | Boundary: minimum |
| 1 | Single entry | Boundary: min useful |
| 3 | Small subset | Nominal |
| 100 | Exceeds available releases | Boundary: max (capped by API data) |
| -1 | Negative integer | Invalid: exit 1 (u64 adapter parse failure) |
| `abc` | Non-integer string | Invalid: type mismatch |

Boundary set: 0, 1, 10 (default), 66 (current release count), 100 (API limit).

### Factor 2: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare: `{version}  {date}` only | Minimum output |
| 1 | Version + date + summary per line | Nominal |
| 2 | Full changelog with `##` headers | Maximum detail |
| `abc` | Non-integer string | Invalid: type mismatch |

### Factor 3: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON array output | Alternate valid |
| `xml` | Unrecognized value | Invalid: rejected |
| `JSON` | Wrong case (case-sensitive) | Invalid: rejected |
| (empty) | Empty string value | Invalid: rejected |

### Factor 4: Network availability (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| available | curl succeeds, data returned | Happy path |
| unavailable | curl fails, no data | Failure: exit 2 |

### Factor 5: HOME environment (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal HOME, cache path accessible | Happy path |
| empty | HOME unset or empty | Failure: exit 2 |

### Factor 6: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 7: Parameter syntax

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| correct | `param::value` style | Happy path |
| flag | `--param` style | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Category |
|----|-------------|-----|------|---------|----------|
| IT-1 | `.version.history` defaults → exits 0 (network permitting) | P | 0/2 | F1=absent, F2=absent, F3=absent, F4=any | Basic Invocation |
| IT-2 | `count::3` → output has ≤3 version entries | P | 0 | F1=3 | Count Param |
| IT-3 | `count::0` → empty output, no version lines | P | 0 | F1=0 | Count Boundary |
| IT-4 | `v::0` → bare `{version}  {date}` lines, no summaries | P | 0 | F2=0 | Verbosity |
| IT-5 | `v::1` explicit → version + date + summary per line | P | 0 | F2=1 | Verbosity |
| IT-6 | `v::2` → full changelog with `##` headers per version | P | 0 | F2=2 | Verbosity |
| IT-7 | `format::json` → valid JSON array with version/date/summary fields | P | 0 | F3=json | Format |
| IT-8 | `count::1 format::json` → JSON array with exactly 1 element | P | 0 | F1=1, F3=json | Pairwise: count x format |
| IT-9 | `count::1 v::0` → exactly 1 bare line | P | 0 | F1=1, F2=0 | Pairwise: count x verbosity |
| IT-10 | `count::1 v::2` → single full changelog block | P | 0 | F1=1, F2=2 | Pairwise: count x verbosity |
| IT-11 | Default count ≤10 entries (verify default value) | P | 0 | F1=absent | Default Behavior |
| IT-12 | `count::100` → all available releases, capped by data | P | 0 | F1=100 | Count Boundary |
| IT-13 | Idempotency: two consecutive calls produce identical output | P | 0 | F1=1 | Stability |
| IT-14 | Param order: `count::3 v::0` = `v::0 count::3` | P | 0 | F1=3, F2=0 | Commutativity |
| IT-15 | `count::0 format::json` → empty JSON array `[]` | P | 0 | F1=0, F3=json | Empty vs Error |
| IT-26 | UTF-8 non-ASCII chars in release body preserved intact | P | 0 | F3=absent, F2=2 | Bug Fix |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Category |
|----|-------------|-----|------|---------|----------|
| IT-16 | `format::xml` → exit 1, unknown format | N | 1 | F3=xml | Format Validation |
| IT-17 | `format::JSON` (uppercase) → exit 1, case-sensitive | N | 1 | F3=JSON | Format Validation |
| IT-18 | `format::` (empty value) → exit 1 | N | 1 | F3="" | Format Validation |
| IT-19 | Unknown param `bogus::x` → exit 1 | N | 1 | F6=present | Param Validation |
| IT-20 | Network unavailable → exit 2, stderr contains error | N | 2 | F4=unavailable | Error Handling |
| IT-21 | HOME empty → exit 2 | N | 2 | F5=empty | Environment |
| IT-22 | `count::-1` (negative) → parse error → exit 1 | N | 1 | F1=-1 | Type Validation |
| IT-23 | `v::abc` → exit 1, type mismatch for Integer | N | 1 | F2=invalid | Type Validation |
| IT-24 | `count::abc` → exit 1, type mismatch for Integer | N | 1 | F1=invalid | Type Validation |
| IT-25 | `--verbose` flag-style → exit 1 | N | 1 | F7=flag-style | Syntax Validation |

### Summary

- **Total:** 26 tests (16 positive, 10 negative)
- **Negative ratio:** 38.5% — supplemented by bug-fix IT-26 (positive); meets coverage intent
- **TC range:** IT-1 to IT-26

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (with or without output) | IT-1 through IT-15, IT-26 |
| 1 | Invalid arguments (format, type, unknown param, syntax) | IT-16 through IT-19, IT-22 through IT-25 |
| 2 | Operational failure (network, HOME) | IT-20, IT-21 |

### Empty vs Error Distinction

| State | Exit | Output | Tests |
|-------|------|--------|-------|
| Valid non-empty | 0 | Version entries on stdout | IT-1, IT-2, IT-5, etc. |
| Valid empty | 0 | Empty stdout (text) or `[]` (json) | IT-3 (text), IT-15 (json) |
| Error | 2 | Error message on stderr | IT-20, IT-21 |

### Pairwise Coverage: F1 (count) x F2 (verbosity)

| count \ v | absent | 0 | 1 | 2 |
|-----------|--------|---|---|---|
| absent | IT-1 | IT-4 | IT-5 | IT-6 |
| 0 | IT-3 | pruned | pruned | pruned |
| 1 | — | IT-9 | (IT-8 implicit) | IT-10 |
| 3 | IT-2 | IT-14 | — | — |
| 100 | IT-12 | pruned | pruned | pruned |
| -1 | IT-22 (exit 1) | — | — | — |

**Pruned with justification:**
- (0, 0/1/2): `count::0` produces zero entries; verbosity formats entries but there are none. Output is always empty regardless of verbosity.
- (100, 0/1/2): `count::100` exercises the data-volume boundary; verbosity formatting is an independent dimension already covered by (absent, 0/1/2).
- (-1, v/format): IT-22 exits 1 at adapter parse (u64 rejects negative); no handler output to cover.

### Pairwise Coverage: F1 (count) x F3 (format)

| count \ format | absent | json |
|----------------|--------|------|
| absent | IT-1 | IT-7 |
| 0 | IT-3 | IT-15 |
| 1 | — | IT-8 |
| 3 | IT-2 | pruned |
| 100 | IT-12 | pruned |

**Pruned:** (3, json), (100, json) — JSON array structure independent of entry count; covered by (absent, json) + (1, json) + (0, json).

### Pairwise Coverage: F2 (verbosity) x F3 (format)

| v \ format | absent | json |
|------------|--------|------|
| absent | IT-1 | IT-7 |
| 0 | IT-4 | pruned |
| 1 | IT-5 | pruned |
| 2 | IT-6 | pruned |

**Pruned:** (0/1/2, json) — `format::json` produces identical JSON array regardless of verbosity level. JSON output ignores verbosity by design (consistent with `.version.list` sibling).

### Error Path Completeness

| Error Source | Error Message Pattern | Exit | Test(s) |
|-------------|----------------------|------|---------|
| `OutputOptions::from_cmd` | "unknown format '{other}': expected text or json" | 1 | IT-16, IT-17, IT-18 |
| `fetch_releases_json` | "failed to fetch release history" | 2 | IT-20 |
| `require_claude_paths` | "HOME environment variable not set" | 2 | IT-21 |
| Unilang adapter | Type mismatch for Integer param | 1 | IT-22, IT-23, IT-24 |
| Unilang adapter | Unknown parameter rejected | 1 | IT-19 |
| Unilang adapter | Flag-style syntax rejected | 1 | IT-25 |

### Sibling Parity: `.version.list`

| .version.list test | Coverage dimension | .version.history equivalent |
|--------------------|-------------------|-----------------------------|
| TC-115 (exit 0) | Basic invocation | IT-1 |
| TC-116/117 (content presence) | Content validation | IT-5 (summary) |
| TC-118 (v::0 minimal) | Bare output | IT-4 |
| TC-119 (v::1 descriptions) | Labeled output | IT-5 |
| TC-120 (idempotency) | Repeat stability | IT-13 |
| TC-121 (format::json array) | JSON structure | IT-7 |
| TC-124 (JSON field presence) | JSON fields | IT-7, IT-8 |

---

## Test Case Details

---

### IT-1: Default invocation exits 0

- **Given:** Network available; HOME set.
- **When:**
  `cm .version.history`
  **Expected:** Exit 0 with version history on stdout. If network unavailable, exit 2 is acceptable.
- **Then:** with output, or exit 2 with error on stderr
- **Exit:** 0

---

### IT-2: count::3 limits output to 3 entries

- **Given:** Network available.
- **When:**
  `cm .version.history count::3`
  **Expected:** At most 3 version entries in output.
- **Then:** ≤3 version lines
- **Exit:** 0

---

### IT-3: count::0 produces empty output

- **Given:** Network available (but count::0 should produce empty output even before fetch, or after fetch with zero entries selected).
- **When:**
  `cm .version.history count::0`
  **Expected:** Exit 0; stdout is empty.
- **Then:** empty stdout
- **Exit:** 0

---

### IT-4: v::0 produces bare version+date lines

- **Given:** Network available.
- **When:**
  `cm .version.history v::0 count::3`
  **Expected:** Lines matching `{semver}  {YYYY-MM-DD}` pattern, no summaries or labels.
- **Then:** bare format, no summaries
- **Exit:** 0

---

### IT-5: v::1 shows version + date + summary

- **Given:** Network available.
- **When:**
  `cm .version.history v::1 count::3`
  **Expected:** Each line has version, date, and a changelog summary.
- **Then:** summary text present
- **Exit:** 0

---

### IT-6: v::2 shows full changelog with ## headers

- **Given:** Network available.
- **When:**
  `cm .version.history v::2 count::2`
  **Expected:** Output contains `##` markdown headers and `- ` bullet lines.
- **Then:** `##` headers and `- ` bullets present
- **Exit:** 0

---

### IT-7: format::json produces valid JSON array

- **Given:** Network available.
- **When:**
  `cm .version.history format::json count::3`
  **Expected:** JSON array where each element has `version`, `date`, `summary` fields.
- **Then:** valid JSON array with required fields
- **Exit:** 0

---

### IT-8: count::1 format::json produces single-element array

- **Given:** Network available.
- **When:**
  `cm .version.history count::1 format::json`
  **Expected:** JSON array with exactly 1 object element.
- **Then:** single-element JSON array
- **Exit:** 0

---

### IT-9: count::1 v::0 produces single bare line

- **Given:** Network available.
- **When:**
  `cm .version.history count::1 v::0`
  **Expected:** Exactly 1 line of bare `{version}  {date}`.
- **Then:** exactly 1 bare line
- **Exit:** 0

---

### IT-10: count::1 v::2 produces single changelog block

- **Given:** Network available.
- **When:**
  `cm .version.history count::1 v::2`
  **Expected:** One `##` header block with changelog bullets for a single release.
- **Then:** single `##` block
- **Exit:** 0

---

### IT-11: Default count is 10

- **Given:** Network available; API has 66+ releases.
- **When:**
  `cm .version.history`
  **Expected:** At most 10 version entries in output.
- **Then:** ≤10 version entries
- **Exit:** 0

---

### IT-12: count::100 returns all available releases

- **Given:** Network available.
- **When:**
  `cm .version.history count::100 v::0`
  **Expected:** All available releases (currently ~66), each on its own line.
- **Then:** >10 entries returned
- **Exit:** 0

---

### IT-13: Idempotency — two calls produce identical output

- **Given:** Network available.
- **When:**
  Run `.version.history count::1` twice.
  **Expected:** Both outputs are byte-identical.
- **Then:** Exit codes equal; outputs equal
- **Exit:** 0

---

### IT-14: Parameter order independence

- **Given:** Network available.
- **When:**
  Run with both parameter orderings.
  **Expected:** Identical output regardless of parameter order.
- **Then:** Exit codes equal; outputs equal
- **Exit:** 0

---

### IT-15: count::0 format::json produces empty array

- **Given:** Network available.
- **When:**
  `cm .version.history count::0 format::json`
  **Expected:** Exit 0; output is `[]` (empty JSON array).
- **Then:** `[]` output
- **Exit:** 0

---

### IT-16: format::xml exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history format::xml`
  **Expected:** Exit 1; stderr mentions unknown format.
- **Then:** descriptive error on stderr
- **Exit:** 1

---

### IT-17: format::JSON (uppercase) exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history format::JSON`
  **Expected:** Exit 1; same error as unknown format.
- **Then:** see spec
- **Exit:** 1

---

### IT-18: format:: (empty value) exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history format::`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-19: Unknown parameter exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history bogus::x`
  **Expected:** Exit 1; error about unknown parameter.
- **Then:** see spec
- **Exit:** 1

---

### IT-20: Network unavailable exits 2

- **Given:** Network unavailable or curl unreachable.
- **When:**
  `cm .version.history`
  **Expected:** Exit 2; stderr contains "failed to fetch" or similar.
- **Then:** error on stderr.
**Note:** Manual verification test — cannot be reliably triggered in standard CI
- **Exit:** 2

---

### IT-21: HOME empty exits 2

- **Given:** Override HOME to empty string.
- **When:**
  `cm .version.history` with `HOME=""`
  **Expected:** Exit 2; error about HOME.
- **Then:** see spec
- **Exit:** 2

---

### IT-22: count::-1 (negative) → parse error → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.history count::-1`
  **Expected:** Exit 1; adapter rejects negative as type error.
- **Then:** see spec
- **Exit:** 1

---

### IT-23: v::abc (non-integer) exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history v::abc`
  **Expected:** Exit 1; type mismatch error.
- **Then:** see spec
- **Exit:** 1

---

### IT-24: count::abc (non-integer) exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history count::abc`
  **Expected:** Exit 1; type mismatch error.
- **Then:** see spec
- **Exit:** 1

---

### IT-25: --verbose flag-style exits 1

- **Given:** clean environment
- **When:**
  `cm .version.history --verbose`
  **Expected:** Exit 1; error mentions `param::value` syntax.
- **Then:** see spec
- **Exit:** 1

---

### IT-26: UTF-8 non-ASCII body characters preserved

- **Given:** `HOME=<tmp>`; write `version_history_cache.json` to `<tmp>/.claude/.transient/` containing a release body with raw em-dash (U+2014) and right-quote (U+2019) bytes (not `\uXXXX` escapes).
- **When:**
  `cm .version.history v::2 count::1`
  **Expected:** Exit 0; stdout contains the em-dash and right-quote characters intact; stdout does not contain garbled U+00E2 (`â`).
- **Then:** all three character assertions pass
- **Exit:** 0

---

## Test Implementation Strategy

### Network-Conditional Helper

```rust
fn skip_if_no_network( out : &std::process::Output ) -> bool
{
  if out.status.code() == Some( 2 )
  {
    let err = String::from_utf8_lossy( &out.stderr );
    if err.contains( "failed to fetch" ) { return true; }
  }
  false
}
```

Tests requiring successful network fetch:
1. Run the command
2. If `skip_if_no_network()` returns true → return (test passes vacuously)
3. Otherwise assert exit 0 and verify output format

### Test Isolation Patterns

| Pattern | Tests | Method |
|---------|-------|--------|
| No isolation needed | IT-16 to IT-19, IT-22 to IT-25 | Fails at adapter/validation, before state access |
| Network-conditional | IT-1 to IT-15 | `skip_if_no_network()` helper |
| HOME isolation | IT-21, IT-26 | `run_cm_with_env(&[...], &[("HOME", "<tmp>")])` |
| Manual verification | IT-20 | Cannot reliably trigger in CI |

### Categories Summary

| Category | Tests | Count |
|----------|-------|-------|
| Basic Invocation | IT-1 | 1 |
| Count Param | IT-2 | 1 |
| Count Boundary | IT-3, IT-12 | 2 |
| Verbosity | IT-4, IT-5, IT-6 | 3 |
| Format | IT-7 | 1 |
| Pairwise: count x format | IT-8, IT-15 | 2 |
| Pairwise: count x verbosity | IT-9, IT-10 | 2 |
| Default Behavior | IT-11 | 1 |
| Stability | IT-13 | 1 |
| Commutativity | IT-14 | 1 |
| Format Validation | IT-16, IT-17, IT-18 | 3 |
| Param Validation | IT-19 | 1 |
| Error Handling | IT-20 | 1 |
| Environment | IT-21 | 1 |
| Type Validation | IT-22, IT-23, IT-24 | 3 |
| Syntax Validation | IT-25 | 1 |
| Bug Fix | IT-26 | 1 |
