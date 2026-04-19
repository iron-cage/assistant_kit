# Test: `.help`

Integration test planning for the `.help` command. See [commands.md](../../../../../docs/cli/commands.md#command--2-help) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `.help` lists all registered visible commands | Content |
| IT-2 | `.help` excludes hidden commands (only bare `.`) | Visibility |
| IT-3 | `.help` shows usage line with binary name | Content |
| IT-4 | `.help` exits 0 | Exit Code |
| IT-5 | `.help` output includes command purposes | Content |
| IT-6 | `.help` output includes parameter hints per command | Content |
| IT-7 | `.help` IS visible in its own listing | Visibility |
| IT-8 | `.help` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Content: 4 tests
- Visibility: 2 tests
- Exit Code: 1 test
- Stability: 1 test

**Total:** 8 integration tests

---

### IT-1: `.help` lists all registered visible commands

**Goal:** Confirm the help output includes all 10 registered visible commands by name.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** Output contains all 10 registered command names.
**Verification:**
- Capture stdout
- Assert stdout contains the string `.account.list`
- Assert stdout contains the string `.account.status`
- Assert stdout contains the string `.account.save`
- Assert stdout contains the string `.account.switch`
- Assert stdout contains the string `.account.delete`
- Assert stdout contains the string `.token.status`
- Assert stdout contains the string `.paths`
- Assert stdout contains the string `.usage`
- Assert stdout contains the string `.credentials.status`
- Assert stdout contains the string `.account.limits`
**Pass Criteria:** Exit 0; all 10 registered command names present in output.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-2: `.help` excludes hidden commands

**Goal:** Confirm that the hidden command `.` does not appear in the listing. Note: `.help` IS visible (unilang auto-registers it as a non-hidden command).
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** The listing does not contain a bare `.` entry. The 10 registered commands plus `.help` itself (11 total) are all visible.
**Verification:**
- Capture stdout
- Extract lines that start with `.` after trimming whitespace
- Assert no extracted line has `.` as its sole command token (i.e. bare dot is absent)
- Assert bare `.` does not appear as a standalone entry distinct from `.account.*`, `.token.*`, etc.
**Pass Criteria:** Exit 0; bare `.` absent from listing; `.help` visible.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-3: `.help` shows usage line with binary name

**Goal:** Confirm the output contains a usage line that references the binary name `clp`.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** Output contains a line matching `Usage: clp <command> [params]`.
**Verification:**
- Capture stdout
- Assert stdout contains the string `Usage: clp`
- Assert the usage line includes `<command>` placeholder syntax
**Pass Criteria:** Exit 0; usage line with `clp` binary name is present.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-4: `.help` exits 0

**Goal:** Confirm the `.help` command exits with success status code 0.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** Help text printed to stdout; exit code 0.
**Verification:**
- Run command and capture exit code
- Assert exit code equals 0
**Pass Criteria:** Exit 0.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-5: `.help` output includes command purposes

**Goal:** Confirm that each listed command is accompanied by its purpose description string.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** Each command entry in the listing is followed by its purpose text as defined in the spec.
**Verification:**
- Capture stdout
- Assert stdout contains `List all saved accounts` (purpose of `.account.list`)
- Assert stdout contains `Show account name` or `account name and token` (purpose of `.account.status`)
- Assert stdout contains `Save current credentials` (purpose of `.account.save`)
- Assert stdout contains `Switch active account` (purpose of `.account.switch`)
- Assert stdout contains `Delete a saved account` (purpose of `.account.delete`)
- Assert stdout contains `Show active OAuth token expiry` (purpose of `.token.status`)
- Assert stdout contains `Show all resolved` (purpose of `.paths`)
- Assert stdout contains `7-day token usage` or `token usage` (purpose of `.usage`)
- Assert stdout contains `live credentials` or `credential` (purpose of `.credentials.status`)
- Assert stdout contains `rate-limit` or `limits` (purpose of `.account.limits`)
**Pass Criteria:** Exit 0; all 10 command purpose descriptions present in output.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-6: `.help` output includes parameter hints per command

**Goal:** Confirm that commands with parameters show parameter hints in the help listing.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** Commands that accept parameters show parameter names or placeholders (e.g., `name::`, `v::`, `format::`, `dry::`, `threshold::`).
**Verification:**
- Capture stdout
- Assert the `.account.save` line or its description references its parameters (e.g., `name::`)
- Assert the `.token.status` line or its description references its parameters (e.g., `threshold::`)
- Assert commands with parameters are distinguishable from parameterless commands
**Pass Criteria:** Exit 0; parameter hints visible for commands that accept parameters.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-7: `.help` IS visible in its own listing

**Goal:** Confirm `.help` appears in its own command listing. Unilang auto-registers `.help` as a non-hidden command, so it must appear alongside the 10 user-registered commands (11 total visible).
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help`
**Expected Output:** The listing contains `.help` as a command entry; 11 total visible commands.
**Verification:**
- Capture stdout
- Extract lines that start with `.` after trimming whitespace
- Assert at least one extracted line starts with `.help` as a command name column entry
**Pass Criteria:** Exit 0; `.help` appears in the listing as a visible command.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)

---

### IT-8: `.help` output is stable across repeated invocations

**Goal:** Confirm that running `.help` multiple times produces identical output, proving deterministic behavior.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .help` (run 3 times)
**Expected Output:** All 3 invocations produce identical stdout.
**Verification:**
- Run `clp .help` three times, capturing stdout each time
- Assert output from run 1 == output from run 2
- Assert output from run 2 == output from run 3
**Pass Criteria:** Exit 0 on all runs; all 3 outputs are byte-identical.
**Source:** [commands.md — .help](../../../../../docs/cli/commands.md#command--2-help)
