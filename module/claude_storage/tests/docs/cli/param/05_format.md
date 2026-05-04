# Parameter :: `format::`

Edge case tests for the `format::` parameter. Tests validate enum parsing, default, and output shape per format.

**Source:** [params.md#parameter--5-format](../../../../docs/cli/params.md#parameter--5-format) | [types.md#exportformat](../../../../docs/cli/types.md#exportformat)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "markdown" accepted | Enum Values |
| EC-2 | Value "json" accepted | Enum Values |
| EC-3 | Value "text" accepted | Enum Values |
| EC-4 | Value "MARKDOWN" accepted (case-insensitive) | Case Insensitivity |
| EC-5 | Invalid value "html" rejected with error | Error Handling |
| EC-6 | Invalid value "pdf" rejected with error | Error Handling |
| EC-7 | Omitted defaults to "markdown" | Default |

## Test Coverage Summary

- Enum Values: 3 tests (EC-1, EC-2, EC-3)
- Case Insensitivity: 1 test (EC-4)
- Error Handling: 2 tests (EC-5, EC-6)
- Default: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value "markdown" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::markdown output::/tmp/test-out/session.md`
- **Then:** No error; file written at `/tmp/test-out/session.md` with markdown content (headings, message formatting).; output file exists with markdown structure
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value "json" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::json output::/tmp/test-out/session.json`
- **Then:** No error; file written at `/tmp/test-out/session.json` with a JSON array.; output file contains valid JSON array
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value "text" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::text output::/tmp/test-out/session.txt`
- **Then:** No error; file written at `/tmp/test-out/session.txt` with plain text content (no `#` headings or `**bold**`).; output file exists with plain text content
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Value "MARKDOWN" accepted (case-insensitive)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::MARKDOWN output::/tmp/test-out/session.md`
- **Then:** No error; file written with markdown content identical to using lowercase `format::markdown`.; output file exists (same result as EC-1)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Invalid value "html" rejected with error

- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic format::html output::/tmp/test-out/session.html`
- **Then:** stderr contains `format must be markdown|json|text, got html`; error message `format must be markdown|json|text, got html`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Invalid value "pdf" rejected with error

- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic format::pdf output::/tmp/test-out/session.pdf`
- **Then:** stderr contains `format must be markdown|json|text, got pdf`; error message `format must be markdown|json|text, got pdf`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Omitted defaults to "markdown"

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::/tmp/test-out/session.md`
- **Then:** No error; file written with markdown content (same as EC-1 with `format::markdown`).; output file contains markdown content (default applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
