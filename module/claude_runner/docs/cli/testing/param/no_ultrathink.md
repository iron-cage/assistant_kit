# Test: `--no-ultrathink`

Edge case coverage for the `--no-ultrathink` parameter. See [params.md](../../params.md#parameter--14---no-ultrathink) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | Message → `"\n\nultrathink"` suffix present by default (default-on) | Default Behavior |
| TC-02 | `--no-ultrathink "msg"` → message sent verbatim (no suffix) | Opt-Out |
| TC-03 | Message already ending with `"ultrathink"` → not double-suffixed (idempotent guard) | Idempotent Guard |
| TC-04 | `--no-ultrathink` without message → accepted, no error | Edge Case |
| TC-05 | `--help` output contains `--no-ultrathink` | Documentation |

## Test Coverage Summary

- Default Behavior: 1 test
- Opt-Out: 1 test
- Idempotent Guard: 1 test
- Edge Case: 1 test
- Documentation: 1 test

**Total:** 5 edge cases

---

### TC-01: Message → `"\n\nultrathink"` suffix present by default

**Goal:** Without `--no-ultrathink`, `clr` appends `"\n\nultrathink"` to the message before forwarding to claude. This is the default-on behavior.
**Setup:** None.
**Command:** `clr --dry-run "Fix the auth bug"`
**Expected Output:** Command line contains the message followed by `\n\nultrathink` — not bare `"Fix the auth bug"`.
**Verification:** `output.contains("Fix the auth bug")` and `output.contains("ultrathink")` as suffix; `!output.contains("\"ultrathink Fix the auth bug\"")`.
**Pass Criteria:** Exit 0; ultrathink suffix present in assembled command.
**Source:** [params.md — --no-ultrathink](../../params.md#parameter--14---no-ultrathink), [invariant/001_default_flags.md](../../../invariant/001_default_flags.md)

---

### TC-02: `--no-ultrathink "msg"` → message sent verbatim

**Goal:** `--no-ultrathink` suppresses the ultrathink suffix; the message is forwarded exactly as provided.
**Setup:** None.
**Command:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
**Expected Output:** Command line contains `"Fix the auth bug"` — not followed by `\n\nultrathink`.
**Verification:** `output.contains("\"Fix the auth bug\"")` and `!output.contains("ultrathink")` (no suffix present).
**Pass Criteria:** Exit 0; message verbatim; ultrathink suffix absent.
**Source:** [params.md — --no-ultrathink](../../params.md#parameter--14---no-ultrathink)

---

### TC-03: Message already ending with `"ultrathink"` → not double-suffixed

**Goal:** When the message already ends with `"ultrathink"`, the runner's idempotent guard prevents a second suffix from being added.
**Setup:** None.
**Command:** `clr --dry-run` with a message that ends with `"ultrathink"` (e.g., `"fix the bug\n\nultrathink"` in Rust string literal form)
**Expected Output:** Assembled command contains the message unchanged — NOT with an extra `\n\nultrathink` appended.
**Verification:** Output contains exactly one occurrence of `"ultrathink"` at end; does NOT contain `"ultrathink\n\nultrathink"` or `"ultrathinkulthrathink"`.
**Pass Criteria:** Exit 0; message unchanged (single suffix, no double-append).
**Source:** [params.md — --no-ultrathink (idempotent guard)](../../params.md#parameter--14---no-ultrathink)

---

### TC-04: `--no-ultrathink` without message → accepted, no error

**Goal:** `--no-ultrathink` with no message is valid; bare `clr` still opens the interactive REPL without error.
**Setup:** None.
**Command:** `clr --dry-run --no-ultrathink`
**Expected Output:** Exit 0; command assembled normally (no message arg).
**Verification:** Exit 0; last line is bare `claude --dangerously-skip-permissions --chrome -c` (no message suffix).
**Pass Criteria:** Exit 0; no rejection; bare command output present.
**Source:** [params.md — --no-ultrathink (applies only to message-bearing invocations)](../../params.md#parameter--14---no-ultrathink)

---

### TC-05: `--help` lists `--no-ultrathink`

**Goal:** `--no-ultrathink` is documented in help output so users can discover how to disable the default suffix.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Stdout contains `--no-ultrathink`.
**Verification:** `output.contains("--no-ultrathink")`.
**Pass Criteria:** Exit 0; flag present in help.
**Source:** [commands.md — help](../../commands.md#command--2-help)
