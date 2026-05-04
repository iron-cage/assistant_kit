# Test: `--no-ultrathink`

Edge case coverage for the `--no-ultrathink` parameter. See [params.md](../../../../docs/cli/params.md#parameter--14---no-ultrathink) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Message → `"\n\nultrathink"` suffix present by default (default-on) | Default Behavior |
| EC-2 | `--no-ultrathink "msg"` → message sent verbatim (no suffix) | Opt-Out |
| EC-3 | Message already ending with `"ultrathink"` → not double-suffixed (idempotent guard) | Idempotent Guard |
| EC-4 | `--no-ultrathink` without message → accepted, no error | Edge Case |
| EC-5 | `--help` output contains `--no-ultrathink` | Documentation |
| EC-6 | `--no-ultrathink` with empty message `""` → no suffix appended | Edge Case |

## Test Coverage Summary

- Default Behavior: 1 test
- Opt-Out: 1 test
- Idempotent Guard: 1 test
- Edge Case: 2 tests
- Documentation: 1 test

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Message → `"\n\nultrathink"` suffix present by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the auth bug"`
- **Then:** Command line contains the message followed by `\n\nultrathink` — not bare `"Fix the auth bug"`.; ultrathink suffix present in assembled command
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-2: `--no-ultrathink "msg"` → message sent verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
- **Then:** Command line contains `"Fix the auth bug"` — not followed by `\n\nultrathink`.; message verbatim; ultrathink suffix absent
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink)

---

### EC-3: Message already ending with `"ultrathink"` → not double-suffixed

- **Given:** clean environment
- **When:** `clr --dry-run` with a message that ends with `"ultrathink"` (e.g., `"fix the bug\n\nultrathink"` in Rust string literal form)
- **Then:** Assembled command contains the message unchanged — NOT with an extra `\n\nultrathink` appended.; message unchanged (single suffix, no double-append)
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink (idempotent guard)](../../../../docs/cli/params.md#parameter--14---no-ultrathink)

---

### EC-4: `--no-ultrathink` without message → accepted, no error

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink`
- **Then:** Exit 0; command assembled normally (no message arg).; no rejection; bare command output present
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink (applies only to message-bearing invocations)](../../../../docs/cli/params.md#parameter--14---no-ultrathink)

---

### EC-5: `--help` lists `--no-ultrathink`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-ultrathink`.; flag present in help
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### EC-6: `--no-ultrathink` with empty message `""` → no suffix appended

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink ""`
- **Then:** Assembled command contains the empty string argument without any appended `ultrathink` suffix
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink)
