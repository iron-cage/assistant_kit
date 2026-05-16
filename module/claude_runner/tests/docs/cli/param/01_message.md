# Parameter :: `[MESSAGE]`

Edge case tests for the positional message argument. Tests validate word-joining, empty handling, and interaction with --print.

**Source:** [01_message.md](../../../../docs/cli/param/01_message.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Single-word message → forwarded verbatim | Behavioral Divergence |
| EC-2 | Multi-word message → words joined with space | Behavioral Divergence |
| EC-3 | No message → interactive REPL mode (no --print) | Default |
| EC-4 | Message + `--interactive` → TTY passthrough, no --print | Interaction |
| EC-5 | Empty string `""` → treated as no message (bare clr) | Boundary Values |
| EC-6 | Message with special chars (quotes, newlines) → forwarded correctly | Boundary Values |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Default: 1 test (EC-3)
- Interaction: 1 test (EC-4)
- Boundary Values: 2 tests (EC-5, EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: Single-word message → forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run FixBug`
- **Then:** Assembled command includes the word `FixBug` as the message argument
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
---

### EC-2: Multi-word message → joined with space

- **Given:** clean environment
- **When:** `clr --dry-run Fix the bug`
- **Then:** Assembled command contains `Fix the bug` as single joined message argument
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
---

### EC-3: No message → interactive REPL (no --print)

- **Given:** clean environment
- **When:** `clr --dry-run`
- **Then:** Assembled command does NOT contain `--print`; REPL mode invoked
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
---

### EC-4: Message + `--interactive` → TTY passthrough

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Assembled command does NOT contain `--print`; TTY passthrough mode
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
---

### EC-5: Empty string `""` → treated as bare clr

- **Given:** clean environment
- **When:** `clr --dry-run ""`
- **Then:** Assembled command has no message argument; behaves like bare `clr`
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
---

### EC-6: Message with special characters → forwarded correctly

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug: auth fails at line 42"`
- **Then:** Full message string forwarded as single argument without truncation or escaping errors
- **Exit:** 0
- **Source:** [01_message.md](../../../../docs/cli/param/01_message.md)
