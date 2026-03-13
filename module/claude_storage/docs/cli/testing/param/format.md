# Parameter :: `format::`

Edge case tests for the `format::` parameter. Tests validate enum parsing, default, and output shape per format.

**Source:** [params.md#parameter--5-format](../../params.md#parameter--5-format) | [types.md#exportformat](../../types.md#exportformat)

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

## Test Cases

### EC-1: Value "markdown" accepted

**Goal:** Verify that `format::markdown` is accepted and produces a human-readable markdown document.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic format::markdown output::/tmp/test-out/session.md`
**Expected Output:** No error; file written at `/tmp/test-out/session.md` with markdown content (headings, message formatting).
**Verification:**
- Exit code is 0
- Output file exists and contains markdown formatting (lines starting with `#` or `**`)
**Pass Criteria:** exit 0 + output file exists with markdown structure
**Source:** [params.md](../../params.md)

### EC-2: Value "json" accepted

**Goal:** Verify that `format::json` is accepted and produces a JSON array of raw session entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic format::json output::/tmp/test-out/session.json`
**Expected Output:** No error; file written at `/tmp/test-out/session.json` with a JSON array.
**Verification:**
- Exit code is 0
- Output file exists and is valid JSON (parseable with `jq .` or similar)
- Top-level structure is a JSON array (`[...]`)
**Pass Criteria:** exit 0 + output file contains valid JSON array
**Source:** [params.md](../../params.md)

### EC-3: Value "text" accepted

**Goal:** Verify that `format::text` is accepted and produces a plain text transcript without markdown markup.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic format::text output::/tmp/test-out/session.txt`
**Expected Output:** No error; file written at `/tmp/test-out/session.txt` with plain text content (no `#` headings or `**bold**`).
**Verification:**
- Exit code is 0
- Output file exists and contains readable text
- Output does not contain markdown heading syntax (`# `)
**Pass Criteria:** exit 0 + output file exists with plain text content
**Source:** [params.md](../../params.md)

### EC-4: Value "MARKDOWN" accepted (case-insensitive)

**Goal:** Verify that enum parsing is case-insensitive and `format::MARKDOWN` is treated identically to `format::markdown`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic format::MARKDOWN output::/tmp/test-out/session.md`
**Expected Output:** No error; file written with markdown content identical to using lowercase `format::markdown`.
**Verification:**
- Exit code is 0
- Output file exists and contains markdown structure
**Pass Criteria:** exit 0 + output file exists (same result as EC-1)
**Source:** [params.md](../../params.md)

### EC-5: Invalid value "html" rejected with error

**Goal:** Verify that `format::html` is rejected with the exact error message `"format must be markdown|json|text, got html"`.
**Setup:** None
**Command:** `clg .export session_id::-default_topic format::html output::/tmp/test-out/session.html`
**Expected Output:** stderr contains `format must be markdown|json|text, got html`
**Verification:**
- Exit code is 1
- stderr contains the exact string `format must be markdown|json|text, got html`
- No output file is created
**Pass Criteria:** exit 1 + error message `format must be markdown|json|text, got html`
**Source:** [params.md](../../params.md)

### EC-6: Invalid value "pdf" rejected with error

**Goal:** Verify that `format::pdf` is rejected with the exact error message `"format must be markdown|json|text, got pdf"`.
**Setup:** None
**Command:** `clg .export session_id::-default_topic format::pdf output::/tmp/test-out/session.pdf`
**Expected Output:** stderr contains `format must be markdown|json|text, got pdf`
**Verification:**
- Exit code is 1
- stderr contains the exact string `format must be markdown|json|text, got pdf`
- No output file is created
**Pass Criteria:** exit 1 + error message `format must be markdown|json|text, got pdf`
**Source:** [params.md](../../params.md)

### EC-7: Omitted defaults to "markdown"

**Goal:** Verify that omitting `format::` causes `.export` to use `markdown` as the default format.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic output::/tmp/test-out/session.md`
**Expected Output:** No error; file written with markdown content (same as EC-1 with `format::markdown`).
**Verification:**
- Exit code is 0
- Output file exists and contains markdown formatting
- Output is structurally identical to the `format::markdown` result
**Pass Criteria:** exit 0 + output file contains markdown content (default applied)
**Source:** [params.md](../../params.md)
