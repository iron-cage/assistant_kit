# Test: `format::`

Edge case coverage for the `format::` parameter. See [params.md](../../params.md#parameter--3-format) and [types.md](../../types.md#type--3-outputformat) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `format::text` produces labeled human-readable output | Valid Format |
| EC-2 | `format::json` produces valid parseable JSON | Valid Format |
| EC-3 | `format::xml` exits 1 (unknown format) | Invalid Format |
| EC-4 | `format::TEXT` accepted (case-insensitive) | Case Handling |
| EC-5 | `format::JSON` accepted (case-insensitive) | Case Handling |
| EC-6 | Duplicate `format::text format::json` — last wins | Last Wins |
| EC-7 | Omitted `format::` defaults to `format::text` | Default |
| EC-8 | `format::json` output parseable by `jq` | JSON Validity |

## Test Coverage Summary

- Valid Format: 2 tests
- Invalid Format: 1 test
- Case Handling: 2 tests
- Last Wins: 1 test
- Default: 1 test
- JSON Validity: 1 test

**Total:** 8 edge cases

---

### EC-1: Valid Format — Text

**Goal:** Confirm that `format::text` produces human-readable labeled output.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::text`
**Expected Output:** Human-readable text with labels, active indicator, and subscription type. Exit 0.
**Verification:**
- Exit code is 0
- Output contains account names with labels (e.g., `<- active`, subscription type)
- Output is not JSON (does not start with `[` or `{`)
**Pass Criteria:** Exit 0; output is human-readable labeled text, not JSON.
**Source:** [params.md -- format::](../../params.md#parameter--3-format)

---

### EC-2: Valid Format — JSON

**Goal:** Confirm that `format::json` produces valid parseable JSON output.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::json`
**Expected Output:** A JSON array of account objects. Exit 0.
**Verification:**
- Exit code is 0
- Output is valid JSON (parseable without error)
- JSON array contains objects with at minimum `name` and `is_active` fields
**Pass Criteria:** Exit 0; output is syntactically valid JSON with expected structure.
**Source:** [params.md -- format::](../../params.md#parameter--3-format)

---

### EC-3: Invalid Format

**Goal:** Confirm that an unsupported format value is rejected.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::xml`
**Expected Output:** Error message containing `invalid format 'xml'` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `invalid format` and mentions `xml`
- Stderr suggests valid options (`text` or `json`)
- No account listing produced on stdout
**Pass Criteria:** Exit 1; unsupported format rejected with descriptive error naming the bad value.
**Source:** [types.md -- OutputFormat](../../types.md#type--3-outputformat)

---

### EC-4: Case Handling — Uppercase TEXT

**Goal:** Confirm that format matching is case-insensitive and accepts `TEXT`.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::TEXT`
**Expected Output:** Same human-readable labeled output as `format::text`. Exit 0.
**Verification:**
- Exit code is 0
- Output matches `format::text` output exactly
- Output is not JSON
**Pass Criteria:** Exit 0; uppercase `TEXT` accepted and produces text output.
**Source:** [types.md -- OutputFormat](../../types.md#type--3-outputformat)

---

### EC-5: Case Handling — Uppercase JSON

**Goal:** Confirm that format matching is case-insensitive and accepts `JSON`.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::JSON`
**Expected Output:** Same JSON output as `format::json`. Exit 0.
**Verification:**
- Exit code is 0
- Output is valid JSON (parseable without error)
- Output matches `format::json` output exactly
**Pass Criteria:** Exit 0; uppercase `JSON` accepted and produces JSON output.
**Source:** [types.md -- OutputFormat](../../types.md#type--3-outputformat)

---

### EC-6: Last Wins

**Goal:** Confirm that when `format::` is specified multiple times, the last occurrence takes precedence.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list format::text format::json`
**Expected Output:** JSON output (matching `format::json` behavior), not text output. Exit 0.
**Verification:**
- Exit code is 0
- Output is valid JSON (parseable without error)
- Output is not human-readable text with labels
**Pass Criteria:** Exit 0; last `format::` value (`json`) takes effect, producing JSON output.
**Source:** [params.md -- format::](../../params.md#parameter--3-format)

---

### EC-7: Default

**Goal:** Confirm that omitting `format::` defaults to `format::text`.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list`
**Expected Output:** Human-readable labeled text output, same as `format::text`. Exit 0.
**Verification:**
- Exit code is 0
- Output contains labels and active indicator (text format)
- Output is not JSON (does not start with `[` or `{`)
- Output matches `format::text` behavior
**Pass Criteria:** Exit 0; default output is text format with labels.
**Source:** [params.md -- format::](../../params.md#parameter--3-format)

---

### EC-8: JSON Validity

**Goal:** Confirm that `format::json` output is fully parseable by `jq` without errors.
**Setup:** At least one saved account exists under `~/.claude/accounts/`. `jq` is available on PATH.
**Command:** `clp .account.list format::json | jq .`
**Expected Output:** Pretty-printed JSON from `jq` with exit 0 from both the pipeline stages.
**Verification:**
- Pipeline exit code is 0
- `jq` does not emit parse errors on stderr
- `jq .` output is well-formed pretty-printed JSON
- Round-trip: `jq -c .` output matches the original `format::json` output (modulo whitespace)
**Pass Criteria:** Exit 0; `jq` successfully parses the JSON output without errors.
**Source:** [params.md -- format::](../../params.md#parameter--3-format)
