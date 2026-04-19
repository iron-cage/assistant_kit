# Test: `.version.history`

Integration test planning for the `.version.history` command. See [commands.md](../../../../../docs/cli/commands.md) for specification.

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
| TC-425 | `.version.history` defaults → exits 0 (network permitting) | P | 0/2 | F1=absent, F2=absent, F3=absent, F4=any | Basic Invocation |
| TC-426 | `count::3` → output has ≤3 version entries | P | 0 | F1=3 | Count Param |
| TC-427 | `count::0` → empty output, no version lines | P | 0 | F1=0 | Count Boundary |
| TC-428 | `v::0` → bare `{version}  {date}` lines, no summaries | P | 0 | F2=0 | Verbosity |
| TC-429 | `v::1` explicit → version + date + summary per line | P | 0 | F2=1 | Verbosity |
| TC-430 | `v::2` → full changelog with `##` headers per version | P | 0 | F2=2 | Verbosity |
| TC-431 | `format::json` → valid JSON array with version/date/summary fields | P | 0 | F3=json | Format |
| TC-432 | `count::1 format::json` → JSON array with exactly 1 element | P | 0 | F1=1, F3=json | Pairwise: count x format |
| TC-433 | `count::1 v::0` → exactly 1 bare line | P | 0 | F1=1, F2=0 | Pairwise: count x verbosity |
| TC-434 | `count::1 v::2` → single full changelog block | P | 0 | F1=1, F2=2 | Pairwise: count x verbosity |
| TC-435 | Default count ≤10 entries (verify default value) | P | 0 | F1=absent | Default Behavior |
| TC-436 | `count::100` → all available releases, capped by data | P | 0 | F1=100 | Count Boundary |
| TC-437 | Idempotency: two consecutive calls produce identical output | P | 0 | F1=1 | Stability |
| TC-438 | Param order: `count::3 v::0` = `v::0 count::3` | P | 0 | F1=3, F2=0 | Commutativity |
| TC-439 | `count::0 format::json` → empty JSON array `[]` | P | 0 | F1=0, F3=json | Empty vs Error |
| TC-450 | UTF-8 non-ASCII chars in release body preserved intact | P | 0 | F3=absent, F2=2 | Bug Fix |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Category |
|----|-------------|-----|------|---------|----------|
| TC-440 | `format::xml` → exit 1, unknown format | N | 1 | F3=xml | Format Validation |
| TC-441 | `format::JSON` (uppercase) → exit 1, case-sensitive | N | 1 | F3=JSON | Format Validation |
| TC-442 | `format::` (empty value) → exit 1 | N | 1 | F3="" | Format Validation |
| TC-443 | Unknown param `bogus::x` → exit 1 | N | 1 | F6=present | Param Validation |
| TC-444 | Network unavailable → exit 2, stderr contains error | N | 2 | F4=unavailable | Error Handling |
| TC-445 | HOME empty → exit 2 | N | 2 | F5=empty | Environment |
| TC-446 | `count::-1` (negative) → parse error → exit 1 | N | 1 | F1=-1 | Type Validation |
| TC-447 | `v::abc` → exit 1, type mismatch for Integer | N | 1 | F2=invalid | Type Validation |
| TC-448 | `count::abc` → exit 1, type mismatch for Integer | N | 1 | F1=invalid | Type Validation |
| TC-449 | `--verbose` flag-style → exit 1 | N | 1 | F7=flag-style | Syntax Validation |

### Summary

- **Total:** 26 tests (16 positive, 10 negative)
- **Negative ratio:** 38.5% — supplemented by bug-fix TC-450 (positive); meets coverage intent
- **TC range:** TC-425 to TC-450

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (with or without output) | TC-425 through TC-439, TC-450 |
| 1 | Invalid arguments (format, type, unknown param, syntax) | TC-440 through TC-443, TC-446 through TC-449 |
| 2 | Operational failure (network, HOME) | TC-444, TC-445 |

### Empty vs Error Distinction

| State | Exit | Output | Tests |
|-------|------|--------|-------|
| Valid non-empty | 0 | Version entries on stdout | TC-425, TC-426, TC-429, etc. |
| Valid empty | 0 | Empty stdout (text) or `[]` (json) | TC-427 (text), TC-439 (json) |
| Error | 2 | Error message on stderr | TC-444, TC-445 |

### Pairwise Coverage: F1 (count) x F2 (verbosity)

| count \ v | absent | 0 | 1 | 2 |
|-----------|--------|---|---|---|
| absent | TC-425 | TC-428 | TC-429 | TC-430 |
| 0 | TC-427 | pruned | pruned | pruned |
| 1 | — | TC-433 | (TC-432 implicit) | TC-434 |
| 3 | TC-426 | TC-438 | — | — |
| 100 | TC-436 | pruned | pruned | pruned |
| -1 | TC-446 (exit 1) | — | — | — |

**Pruned with justification:**
- (0, 0/1/2): `count::0` produces zero entries; verbosity formats entries but there are none. Output is always empty regardless of verbosity.
- (100, 0/1/2): `count::100` exercises the data-volume boundary; verbosity formatting is an independent dimension already covered by (absent, 0/1/2).
- (-1, v/format): TC-446 exits 1 at adapter parse (u64 rejects negative); no handler output to cover.

### Pairwise Coverage: F1 (count) x F3 (format)

| count \ format | absent | json |
|----------------|--------|------|
| absent | TC-425 | TC-431 |
| 0 | TC-427 | TC-439 |
| 1 | — | TC-432 |
| 3 | TC-426 | pruned |
| 100 | TC-436 | pruned |

**Pruned:** (3, json), (100, json) — JSON array structure independent of entry count; covered by (absent, json) + (1, json) + (0, json).

### Pairwise Coverage: F2 (verbosity) x F3 (format)

| v \ format | absent | json |
|------------|--------|------|
| absent | TC-425 | TC-431 |
| 0 | TC-428 | pruned |
| 1 | TC-429 | pruned |
| 2 | TC-430 | pruned |

**Pruned:** (0/1/2, json) — `format::json` produces identical JSON array regardless of verbosity level. JSON output ignores verbosity by design (consistent with `.version.list` sibling).

### Error Path Completeness

| Error Source | Error Message Pattern | Exit | Test(s) |
|-------------|----------------------|------|---------|
| `OutputOptions::from_cmd` | "unknown format '{other}': expected text or json" | 1 | TC-440, TC-441, TC-442 |
| `fetch_releases_json` | "failed to fetch release history" | 2 | TC-444 |
| `require_claude_paths` | "HOME environment variable not set" | 2 | TC-445 |
| Unilang adapter | Type mismatch for Integer param | 1 | TC-446, TC-447, TC-448 |
| Unilang adapter | Unknown parameter rejected | 1 | TC-443 |
| Unilang adapter | Flag-style syntax rejected | 1 | TC-449 |

### Sibling Parity: `.version.list`

| .version.list test | Coverage dimension | .version.history equivalent |
|--------------------|-------------------|-----------------------------|
| TC-115 (exit 0) | Basic invocation | TC-425 |
| TC-116/117 (content presence) | Content validation | TC-429 (summary) |
| TC-118 (v::0 minimal) | Bare output | TC-428 |
| TC-119 (v::1 descriptions) | Labeled output | TC-429 |
| TC-120 (idempotency) | Repeat stability | TC-437 |
| TC-121 (format::json array) | JSON structure | TC-431 |
| TC-124 (JSON field presence) | JSON fields | TC-431, TC-432 |

---

## Test Case Details

### TC-425: Default invocation exits 0

**Goal:** Basic command invocation succeeds when network is available.
**Setup:** Network available; HOME set.
**Command:** `cm .version.history`
**Expected:** Exit 0 with version history on stdout. If network unavailable, exit 2 is acceptable.
**Verification:**
- exit code is 0 or 2
- if exit 0: stdout is non-empty, contains version-like strings
**Isolation:** `skip_if_no_network()` — passes vacuously if no network.
**Pass Criteria:** Exit 0 with output, or exit 2 with error on stderr.

---

### TC-426: count::3 limits output to 3 entries

**Goal:** The `count::` parameter correctly limits the number of displayed releases.
**Setup:** Network available.
**Command:** `cm .version.history count::3`
**Expected:** At most 3 version entries in output.
**Verification:**
- count non-empty lines (v::1 default: one line per version)
- line count ≤ 3
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; ≤3 version lines.

---

### TC-427: count::0 produces empty output

**Goal:** Zero count means no versions displayed. This is valid-empty, not an error.
**Setup:** Network available (but count::0 should produce empty output even before fetch, or after fetch with zero entries selected).
**Command:** `cm .version.history count::0`
**Expected:** Exit 0; stdout is empty.
**Verification:**
- exit code 0
- stdout is empty string
**Isolation:** Network-conditional (may still need fetch to parse, then show 0).
**Pass Criteria:** Exit 0; empty stdout.

---

### TC-428: v::0 produces bare version+date lines

**Goal:** Verbosity 0 shows minimal output suitable for scripting: version and date only.
**Setup:** Network available.
**Command:** `cm .version.history v::0 count::3`
**Expected:** Lines matching `{semver}  {YYYY-MM-DD}` pattern, no summaries or labels.
**Verification:**
- no line contains description text beyond version + date
- each line has two whitespace-separated fields
**Isolation:** Network-conditional; `count::3` to keep output small.
**Pass Criteria:** Exit 0; bare format, no summaries.

---

### TC-429: v::1 shows version + date + summary

**Goal:** Default verbosity shows a one-line summary per release.
**Setup:** Network available.
**Command:** `cm .version.history v::1 count::3`
**Expected:** Each line has version, date, and a changelog summary.
**Verification:**
- each non-empty line has 3+ whitespace-separated sections
- lines are longer than v::0 equivalents (contain summary text)
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; summary text present.

---

### TC-430: v::2 shows full changelog with ## headers

**Goal:** Maximum verbosity shows complete changelog per release.
**Setup:** Network available.
**Command:** `cm .version.history v::2 count::2`
**Expected:** Output contains `##` markdown headers and `- ` bullet lines.
**Verification:**
- output contains `## ` header lines
- output contains `- ` changelog bullets
- multi-line output (significantly more than count entries)
**Isolation:** Network-conditional; `count::2` to keep output manageable.
**Pass Criteria:** Exit 0; `##` headers and `- ` bullets present.

---

### TC-431: format::json produces valid JSON array

**Goal:** JSON format outputs an array of release objects with required fields.
**Setup:** Network available.
**Command:** `cm .version.history format::json count::3`
**Expected:** JSON array where each element has `version`, `date`, `summary` fields.
**Verification:**
- output starts with `[`
- output contains `"version"` field
- output contains `"date"` field
- output contains `"summary"` field
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; valid JSON array with required fields.

---

### TC-432: count::1 format::json produces single-element array

**Goal:** JSON output respects count parameter.
**Setup:** Network available.
**Command:** `cm .version.history count::1 format::json`
**Expected:** JSON array with exactly 1 object element.
**Verification:**
- count occurrences of `"version"` field = 1
- array structure: `[{...}]` with one element
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; single-element JSON array.

---

### TC-433: count::1 v::0 produces single bare line

**Goal:** Pairwise interaction: count limits entries AND verbosity controls format.
**Setup:** Network available.
**Command:** `cm .version.history count::1 v::0`
**Expected:** Exactly 1 line of bare `{version}  {date}`.
**Verification:**
- line count = 1 (excluding trailing newline)
- line matches bare format (no summary text)
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; exactly 1 bare line.

---

### TC-434: count::1 v::2 produces single changelog block

**Goal:** Pairwise interaction: count::1 limits to 1 release but v::2 shows full changelog.
**Setup:** Network available.
**Command:** `cm .version.history count::1 v::2`
**Expected:** One `##` header block with changelog bullets for a single release.
**Verification:**
- exactly 1 `## ` header line
- at least 1 `- ` bullet line
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; single `##` block.

---

### TC-435: Default count is 10

**Goal:** Without `count::`, the default limits output to 10 releases.
**Setup:** Network available; API has 66+ releases.
**Command:** `cm .version.history`
**Expected:** At most 10 version entries in output.
**Verification:**
- count version lines (matching semver pattern) ≤ 10
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; ≤10 version entries.

---

### TC-436: count::100 returns all available releases

**Goal:** Count exceeding available releases returns everything available, not an error.
**Setup:** Network available.
**Command:** `cm .version.history count::100 v::0`
**Expected:** All available releases (currently ~66), each on its own line.
**Verification:**
- exit code 0
- line count > 10 (more than default)
- line count ≤ 100
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; >10 entries returned.

---

### TC-437: Idempotency — two calls produce identical output

**Goal:** Repeated invocations produce identical results (via cache or deterministic fetch).
**Setup:** Network available.
**Command:** Run `.version.history count::1` twice.
**Expected:** Both outputs are byte-identical.
**Verification:**
- stdout of call 1 == stdout of call 2
**Isolation:** Network-conditional; both calls must succeed or both fail.
**Pass Criteria:** Exit codes equal; outputs equal.

---

### TC-438: Parameter order independence

**Goal:** `count::3 v::0` produces same output as `v::0 count::3`.
**Setup:** Network available.
**Command:** Run with both parameter orderings.
**Expected:** Identical output regardless of parameter order.
**Verification:**
- stdout of ordering A == stdout of ordering B
**Isolation:** Network-conditional; both calls must succeed.
**Pass Criteria:** Exit codes equal; outputs equal.

---

### TC-439: count::0 format::json produces empty array

**Goal:** Empty-but-valid JSON output is `[]`, not `{}` or error.
**Setup:** Network available.
**Command:** `cm .version.history count::0 format::json`
**Expected:** Exit 0; output is `[]` (empty JSON array).
**Verification:**
- exit code 0
- output trimmed equals `[]`
**Isolation:** Network-conditional.
**Pass Criteria:** Exit 0; `[]` output.

---

### TC-440: format::xml exits 1

**Goal:** Unrecognized format values are rejected at argument validation.
**Setup:** None (fails before network access).
**Command:** `cm .version.history format::xml`
**Expected:** Exit 1; stderr mentions unknown format.
**Verification:**
- exit code 1
- stderr contains "unknown format" or "expected text or json"
**Isolation:** None needed; deterministic failure.
**Pass Criteria:** Exit 1; descriptive error on stderr.

---

### TC-441: format::JSON (uppercase) exits 1

**Goal:** Format matching is case-sensitive; uppercase is rejected.
**Setup:** None.
**Command:** `cm .version.history format::JSON`
**Expected:** Exit 1; same error as unknown format.
**Verification:**
- exit code 1
**Isolation:** None needed.
**Pass Criteria:** Exit 1.

---

### TC-442: format:: (empty value) exits 1

**Goal:** Empty format string is not valid.
**Setup:** None.
**Command:** `cm .version.history format::`
**Expected:** Exit 1.
**Verification:**
- exit code 1
**Isolation:** None needed.
**Pass Criteria:** Exit 1.

---

### TC-443: Unknown parameter exits 1

**Goal:** Unregistered parameters are rejected by the adapter.
**Setup:** None.
**Command:** `cm .version.history bogus::x`
**Expected:** Exit 1; error about unknown parameter.
**Verification:**
- exit code 1
**Isolation:** None needed.
**Pass Criteria:** Exit 1.

---

### TC-444: Network unavailable exits 2

**Goal:** When curl fails, the command reports a clear error on stderr and exits 2.
**Setup:** Network unavailable or curl unreachable.
**Command:** `cm .version.history`
**Expected:** Exit 2; stderr contains "failed to fetch" or similar.
**Verification:**
- exit code 2
- stderr is non-empty
**Isolation:** Depends on environment. In CI with network, this test may not trigger.
Testing strategy: This test documents expected behavior. In practice, `skip_if_no_network` in other tests validates the error path when network is actually unavailable.
**Pass Criteria:** Exit 2; error on stderr.
**Note:** Manual verification test — cannot be reliably triggered in standard CI.

---

### TC-445: HOME empty exits 2

**Goal:** When HOME is not set, the command cannot determine config/cache paths.
**Setup:** Override HOME to empty string.
**Command:** `cm .version.history` with `HOME=""`
**Expected:** Exit 2; error about HOME.
**Verification:**
- exit code 2
- stderr contains "HOME" or path-related error
**Isolation:** `run_cm_with_env(&[...], &[("HOME", "")])`.
**Pass Criteria:** Exit 2.

---

### TC-446: count::-1 (negative) → parse error → exit 1

**Goal:** Negative values are rejected by the u64 adapter; count:: cannot be negative.
**Setup:** None.
**Command:** `cm .version.history count::-1`
**Expected:** Exit 1; adapter rejects negative as type error.
**Verification:**
- exit code 1
**Isolation:** None needed; adapter rejects before handler.
**Pass Criteria:** Exit 1.

---

### TC-447: v::abc (non-integer) exits 1

**Goal:** Non-integer value for Integer-typed parameter is rejected.
**Setup:** None.
**Command:** `cm .version.history v::abc`
**Expected:** Exit 1; type mismatch error.
**Verification:**
- exit code 1
**Isolation:** None needed; adapter rejects before handler.
**Pass Criteria:** Exit 1.

---

### TC-448: count::abc (non-integer) exits 1

**Goal:** Non-integer value for `count::` Integer parameter is rejected.
**Setup:** None.
**Command:** `cm .version.history count::abc`
**Expected:** Exit 1; type mismatch error.
**Verification:**
- exit code 1
**Isolation:** None needed; adapter rejects before handler.
**Pass Criteria:** Exit 1.

---

### TC-449: --verbose flag-style exits 1

**Goal:** Flag-style arguments are not supported by the adapter; rejected with guidance.
**Setup:** None.
**Command:** `cm .version.history --verbose`
**Expected:** Exit 1; error mentions `param::value` syntax.
**Verification:**
- exit code 1
- stderr mentions expected syntax
**Isolation:** None needed.
**Pass Criteria:** Exit 1.

---

### TC-450: UTF-8 non-ASCII body characters preserved

**Goal:** Multi-byte UTF-8 characters in release body text (e.g. em-dash U+2014, right-quote U+2019) survive the JSON parsing pipeline and appear intact in output.
**Root Cause:** `parse_json_string_value` iterated by byte index and cast each byte to `char`, breaking multi-byte UTF-8 sequences. Fixed by switching to `str::chars()`.
**Setup:** `HOME=<tmp>`; write `version_history_cache.json` to `<tmp>/.claude/.transient/` containing a release body with raw em-dash (U+2014) and right-quote (U+2019) bytes (not `\uXXXX` escapes).
**Command:** `cm .version.history v::2 count::1`
**Expected:** Exit 0; stdout contains the em-dash and right-quote characters intact; stdout does not contain garbled U+00E2 (`â`).
**Verification:**
- exit code 0
- `text.contains('\u{2014}')` — em-dash intact
- `text.contains('\u{2019}')` — right-quote intact
- `!text.contains('\u{00e2}')` — no garbled first byte of em-dash sequence
**Isolation:** HOME isolation via temp dir with pre-written cache (avoids network).
**Pass Criteria:** Exit 0; all three character assertions pass.

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
| No isolation needed | TC-440 to TC-443, TC-446 to TC-449 | Fails at adapter/validation, before state access |
| Network-conditional | TC-425 to TC-439 | `skip_if_no_network()` helper |
| HOME isolation | TC-445, TC-450 | `run_cm_with_env(&[...], &[("HOME", "<tmp>")])` |
| Manual verification | TC-444 | Cannot reliably trigger in CI |

### Categories Summary

| Category | Tests | Count |
|----------|-------|-------|
| Basic Invocation | TC-425 | 1 |
| Count Param | TC-426 | 1 |
| Count Boundary | TC-427, TC-436 | 2 |
| Verbosity | TC-428, TC-429, TC-430 | 3 |
| Format | TC-431 | 1 |
| Pairwise: count x format | TC-432, TC-439 | 2 |
| Pairwise: count x verbosity | TC-433, TC-434 | 2 |
| Default Behavior | TC-435 | 1 |
| Stability | TC-437 | 1 |
| Commutativity | TC-438 | 1 |
| Format Validation | TC-440, TC-441, TC-442 | 3 |
| Param Validation | TC-443 | 1 |
| Error Handling | TC-444 | 1 |
| Environment | TC-445 | 1 |
| Type Validation | TC-446, TC-447, TC-448 | 3 |
| Syntax Validation | TC-449 | 1 |
| Bug Fix | TC-450 | 1 |
