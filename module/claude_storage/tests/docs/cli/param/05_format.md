# Parameter :: `format::`

Edge case tests for the `format::` parameter. Tests validate enum parsing, default, and output shape per format.

**Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md) | [type/03_export_format.md](../../../../docs/cli/type/03_export_format.md)

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

**Behavioral Divergence Pair:** EC-1 (format::markdown) ↔ EC-2 (format::json)

## Test Cases

---

### EC-1: Value "markdown" accepted

- **Commands:** `.export`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::markdown output::/tmp/test-out/session.md`
- **Then:** No error; file written at `/tmp/test-out/session.md` with markdown content (headings, message formatting).; output file exists with markdown structure
- **Exit:** 0
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-2: Value "json" accepted

- **Commands:** `.export`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::json output::/tmp/test-out/session.json`
- **Then:** No error; file written at `/tmp/test-out/session.json` with a JSON array.; output file contains valid JSON array
- **Exit:** 0
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-3: Value "text" accepted

- **Commands:** `.export`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::text output::/tmp/test-out/session.txt`
- **Then:** No error; file written at `/tmp/test-out/session.txt` with plain text content (no `#` headings or `**bold**`).; output file exists with plain text content
- **Exit:** 0
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-4: Value "MARKDOWN" accepted (case-insensitive)

- **Commands:** `.export`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic format::MARKDOWN output::/tmp/test-out/session.md`
- **Then:** No error; file written with markdown content identical to using lowercase `format::markdown`.; output file exists (same result as EC-1)
- **Exit:** 0
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-5: Invalid value "html" rejected with error

- **Commands:** `.export`
- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic format::html output::/tmp/test-out/session.html`
- **Then:** stderr contains `format must be markdown|json|text, got html`; error message `format must be markdown|json|text, got html`
- **Exit:** 1
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-6: Invalid value "pdf" rejected with error

- **Commands:** `.export`
- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic format::pdf output::/tmp/test-out/session.pdf`
- **Then:** stderr contains `format must be markdown|json|text, got pdf`; error message `format must be markdown|json|text, got pdf`
- **Exit:** 1
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)

---

### EC-7: Omitted defaults to "markdown"

- **Commands:** `.export`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::/tmp/test-out/session.md`
- **Then:** No error; file written with markdown content (same as EC-1 with `format::markdown`).; output file contains markdown content (default applied)
- **Exit:** 0
- **Source:** [param/05_format.md](../../../../docs/cli/param/05_format.md)
