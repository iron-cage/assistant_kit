# Test: `.help` / `.` / empty argv

Integration test planning for help output triggers. See [commands.md](../../commands.md#command--1-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `cm .` → help output, exit 0 | Alias |
| TC-02 | `cm` (empty argv) → help output, exit 0 | Empty State |
| TC-03 | `cm .help` → help output, exit 0 | Happy Path |
| TC-04 | `cm .status .help` → `.help` anywhere in argv triggers help, exit 0 | FR-02 |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 1 test
- Empty State: 1 test
- FR-02 (anywhere-in-argv): 1 test

**Total:** 4 tests (all positive — help is pure output with no side effects)

---

### TC-01: `cm .` → help output, exit 0

**Goal:** The bare-dot alias expands to `.help` in adapter.rs and produces help output.
**Setup:** None.
**Command:** `cm .`
**Expected Output:** Contains command listing; mentions `.help`.
**Verification:**
- exit code 0
- stdout contains at least one known command name (e.g., `.status`)
**Pass Criteria:** Exit 0; help output shown.
**Source:** [commands.md — .help](../../commands.md#command--1-help), adapter.rs bare-dot handling

---

### TC-02: `cm` (empty argv) → help output, exit 0

**Goal:** Empty argv is treated identically to `.help` per FR-03.
**Setup:** None.
**Command:** `cm` (no arguments)
**Expected Output:** Same help output as `.help`.
**Verification:**
- exit code 0
- stdout contains at least one known command name (e.g., `.status`)
**Pass Criteria:** Exit 0; help output shown.
**Source:** [commands.md — .help](../../commands.md#command--1-help), [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-03: `cm .help` → help output, exit 0

**Goal:** Explicit `.help` command produces the full help listing.
**Setup:** None.
**Command:** `cm .help`
**Expected Output:** All 12 commands listed; usage line present.
**Verification:**
- exit code 0
- stdout contains `.help`, `.status`, `.version.show`
**Pass Criteria:** Exit 0; help listing complete.
**Source:** [commands.md — .help](../../commands.md#command--1-help), [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-04: `cm .status .help` → `.help` anywhere in argv triggers help, exit 0

**Goal:** `.help` anywhere in argv (not just first position) triggers help output (FR-02).
**Setup:** None.
**Command:** `cm .status .help`
**Expected Output:** Help output (not `.status` output).
**Verification:**
- exit code 0
- stdout contains help listing (e.g., `.version.show`)
- stdout does NOT contain version/session/settings status fields
**Pass Criteria:** Exit 0; help wins over `.status` when `.help` present anywhere.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md), adapter.rs `.help`-anywhere detection
