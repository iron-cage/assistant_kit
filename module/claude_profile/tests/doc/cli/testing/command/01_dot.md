# Test: `.`

Integration test planning for the `.` command. See [commands.md](../../../../../docs/cli/commands.md#command--1-) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `.` produces identical output to `.help` | Delegation |
| IT-2 | `.` exits 0 | Exit Code |
| IT-3 | `.` is hidden from help listing | Visibility |
| IT-4 | `.` output lists all visible commands | Content |
| IT-5 | `.` output excludes `.` and `.help` from listing | Content |
| IT-6 | `.` output includes usage line | Content |
| IT-7 | `.` with trailing unknown param still shows help | Robustness |
| IT-8 | `.` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Delegation: 1 test
- Exit Code: 1 test
- Visibility: 1 test
- Content: 3 tests
- Robustness: 1 test
- Stability: 1 test

**Total:** 8 integration tests

---

### IT-1: `.` produces identical output to `.help`

**Goal:** Confirm the `.` command delegates fully to `.help` and produces byte-identical output.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .` and `clp .help`
**Expected Output:** Both commands produce identical stdout content, character for character.
**Verification:**
- Capture stdout of `clp .` into variable A
- Capture stdout of `clp .help` into variable B
- Assert A == B (byte-equal comparison)
**Pass Criteria:** Exit 0; stdout of `.` is identical to stdout of `.help`.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-2: `.` exits 0

**Goal:** Confirm the `.` command exits with success status code 0.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .`
**Expected Output:** Help text printed to stdout; exit code 0.
**Verification:**
- Run command and capture exit code
- Assert exit code equals 0
**Pass Criteria:** Exit 0.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-3: `.` is hidden from help listing

**Goal:** Confirm the `.` command does not appear in its own help output listing.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .`
**Expected Output:** Help listing that does not contain a line starting with `.` as a standalone command entry.
**Verification:**
- Capture stdout of `clp .`
- Parse the "Commands:" section
- Assert no line in the command list matches the pattern for a bare `.` command entry
- Specifically assert the string `  .   ` (dot followed by spaces as a command column) does not appear
**Pass Criteria:** Exit 0; `.` does not appear as a listed command in the output.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-4: `.` output lists all visible commands

**Goal:** Confirm the `.` output includes all 10 visible commands.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .`
**Expected Output:** Output contains all 10 visible command names: `.account.list`, `.account.status`, `.account.save`, `.account.switch`, `.account.delete`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`.
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
**Pass Criteria:** Exit 0; all 10 visible command names present in output.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-5: `.` output excludes `.` and `.help` from listing

**Goal:** Confirm the hidden command `.` is excluded from the command listing in the output.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .`
**Expected Output:** The "Commands:" section lists exactly 10 entries (all visible commands except the hidden bare `.`). Note: `.help` IS visible and appears in the listing.
**Verification:**
- Capture stdout
- Extract lines from the "Commands:" section (indented command entries)
- Assert no extracted line matches a bare `.` as a standalone command entry (distinct from `.account.*`, `.token.*`, `.paths`)
- Assert exactly 10 command entries are listed
**Pass Criteria:** Exit 0; bare `.` does not appear as a listed command; exactly 10 entries.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-6: `.` output includes usage line

**Goal:** Confirm the output begins with a usage line that includes the binary name.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .`
**Expected Output:** Output contains a line matching `Usage: clp <command> [params]`.
**Verification:**
- Capture stdout
- Assert stdout contains the string `Usage: clp`
**Pass Criteria:** Exit 0; usage line with binary name is present.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-7: `.` with trailing unknown param still shows help

**Goal:** Confirm the `.` command ignores trailing unknown parameters and still produces help output.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp . foo::bar`
**Expected Output:** Help text identical to `clp .` output (unknown parameter silently ignored).
**Verification:**
- Capture stdout of `clp . foo::bar`
- Capture stdout of `clp .`
- Assert both outputs are identical
- Assert exit code is 0
**Pass Criteria:** Exit 0; output identical to bare `.` invocation.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)

---

### IT-8: `.` output is stable across repeated invocations

**Goal:** Confirm that running `.` multiple times produces identical output, proving deterministic behavior.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp .` (run 3 times)
**Expected Output:** All 3 invocations produce identical stdout.
**Verification:**
- Run `clp .` three times, capturing stdout each time
- Assert output from run 1 == output from run 2
- Assert output from run 2 == output from run 3
**Pass Criteria:** Exit 0 on all runs; all 3 outputs are byte-identical.
**Source:** [commands.md — .](../../../../../docs/cli/commands.md#command--1-)
