# Test: `--version` / `-V`

Integration test planning for the `--version` and `-V` meta-flags. See [`src/lib.rs`](../../../../../src/lib.rs) for implementation.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--version` prints binary name and version | Content |
| IT-2 | `-V` alias behaves identically to `--version` | Alias |
| IT-3 | `--version` exits 0 | Exit Code |
| IT-4 | `--version` output goes to stdout only | Output Stream |
| IT-5 | `--version` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Content: 1 test
- Alias: 1 test
- Exit Code: 1 test
- Output Stream: 1 test
- Stability: 1 test

**Total:** 5 integration tests

---

### IT-1: `--version` prints binary name and version

**Goal:** Confirm `--version` outputs the binary name and crate version on a single line.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp --version`
**Expected Output:** A single line containing the binary name and the crate version (e.g., `clp 0.1.0`).
**Verification:**
- Capture stdout
- Assert stdout contains `clp`
- Assert stdout matches pattern `clp \d+\.\d+\.\d+`
- Assert output is a single non-empty line
**Pass Criteria:** Exit 0; output is `clp <version>`.
**Source:** `src/lib.rs::cli::run()`

---

### IT-2: `-V` alias behaves identically to `--version`

**Goal:** Confirm `-V` produces the same output as `--version`.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp -V`
**Expected Output:** Same output as `clp --version`.
**Verification:**
- Run `clp --version`, capture stdout as `A`
- Run `clp -V`, capture stdout as `B`
- Assert `A == B`
**Pass Criteria:** Exit 0; both outputs are byte-identical.
**Source:** `src/lib.rs::cli::run()`

---

### IT-3: `--version` exits 0

**Goal:** Confirm `--version` exits with success status code 0.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp --version`
**Expected Output:** Version line printed to stdout; exit code 0.
**Verification:**
- Run command and capture exit code
- Assert exit code equals 0
**Pass Criteria:** Exit 0.
**Source:** `src/lib.rs::cli::run()`

---

### IT-4: `--version` output goes to stdout only

**Goal:** Confirm version information is written to stdout only; stderr is empty.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp --version`
**Expected Output:** Non-empty stdout; empty stderr.
**Verification:**
- Run command, capture both stdout and stderr
- Assert stdout is non-empty
- Assert stderr is empty
**Pass Criteria:** Exit 0; stderr is empty; stdout is non-empty.
**Source:** `src/lib.rs::cli::run()`

---

### IT-5: `--version` output is stable across repeated invocations

**Goal:** Confirm that running `--version` multiple times produces identical output, proving deterministic behavior.
**Setup:** No special state required. Default environment with `clp` binary on PATH.
**Command:** `clp --version` (run 3 times)
**Expected Output:** All 3 invocations produce identical stdout.
**Verification:**
- Run `clp --version` three times, capturing stdout each time
- Assert output from run 1 == output from run 2
- Assert output from run 2 == output from run 3
**Pass Criteria:** Exit 0 on all runs; all 3 outputs are byte-identical.
**Source:** `src/lib.rs::cli::run()`
