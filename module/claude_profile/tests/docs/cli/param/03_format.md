# Test: `format::`

Edge case coverage for the `format::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--3-format) and [types.md](../../../../docs/cli/types.md#type--3-outputformat) for specification.

### Test Case Index

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
| EC-9 | `fmt::json` alias produces JSON output (same as `format::json`) | Alias |

### Test Coverage Summary

- Valid Format: 2 tests
- Invalid Format: 1 test
- Case Handling: 2 tests
- Last Wins: 1 test
- Default: 1 test
- JSON Validity: 1 test
- Alias: 1 test

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Valid Format — Text

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::text`
- **Then:** Indented key-val blocks with `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines per account. Exit 0.; output is human-readable labeled text, not JSON
- **Exit:** 0
- **Source:** [params.md -- format::](../../../../docs/cli/params.md#parameter--3-format)

---

### EC-2: Valid Format — JSON

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::json`
- **Then:** A JSON array of account objects. Exit 0.; output is syntactically valid JSON with expected structure
- **Exit:** 0
- **Source:** [params.md -- format::](../../../../docs/cli/params.md#parameter--3-format)

---

### EC-3: Invalid Format

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::xml`
- **Then:** Error message containing `invalid format 'xml'` or similar with exit 1.; unsupported format rejected with descriptive error naming the bad value
- **Exit:** 1
- **Source:** [types.md -- OutputFormat](../../../../docs/cli/types.md#type--3-outputformat)

---

### EC-4: Case Handling — Uppercase TEXT

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::TEXT`
- **Then:** Same human-readable labeled output as `format::text`. Exit 0.; uppercase `TEXT` accepted and produces text output
- **Exit:** 0
- **Source:** [types.md -- OutputFormat](../../../../docs/cli/types.md#type--3-outputformat)

---

### EC-5: Case Handling — Uppercase JSON

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::JSON`
- **Then:** Same JSON output as `format::json`. Exit 0.; uppercase `JSON` accepted and produces JSON output
- **Exit:** 0
- **Source:** [types.md -- OutputFormat](../../../../docs/cli/types.md#type--3-outputformat)

---

### EC-6: Last Wins

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts format::text format::json`
- **Then:** JSON output (matching `format::json` behavior), not text output. Exit 0.; last `format::` value (`json`) takes effect, producing JSON output
- **Exit:** 0
- **Source:** [params.md -- format::](../../../../docs/cli/params.md#parameter--3-format)

---

### EC-7: Default

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`.
- **When:** `clp .accounts`
- **Then:** Human-readable labeled text output with indented key-val blocks, same as `format::text`. Exit 0.; default output is text format with labels
- **Exit:** 0
- **Source:** [params.md -- format::](../../../../docs/cli/params.md#parameter--3-format)

---

### EC-8: JSON Validity

- **Given:** At least one saved account exists under `~/.persistent/claude/credential/`. `jq` is available on PATH.
- **When:** `clp .accounts format::json | jq .`
- **Then:** Pretty-printed JSON from `jq` with exit 0 from both pipeline stages.; `jq` successfully parses the JSON output without errors
- **Exit:** 0
- **Source:** [params.md -- format::](../../../../docs/cli/params.md#parameter--3-format)

---

### EC-9: Alias — `fmt::` accepted as `format::`

- **Given:** Empty credential store (no accounts required).
- **When:** `clp .accounts fmt::json` and `clp .token.status fmt::json` (with a valid credentials file for the latter)
- **Then:** `.accounts` returns a JSON array starting with `[`; `.token.status` returns a JSON object starting with `{`. Exit 0 for both.; `fmt::` alias is expanded to `format::` at runtime — not rejected as an unknown parameter
- **Exit:** 0
- **Source:** `tests/cli/cross_cutting_test.rs (e11, e12)`
