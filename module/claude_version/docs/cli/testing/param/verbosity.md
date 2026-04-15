# Test: `v::` / `verbosity::` (verbosity)

Edge case coverage for the `v::` alias and `verbosity::` canonical key. See [params.md](../../params.md) and [types.md](../../types.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-097 | `.status v::0` → 3 bare lines | Explicit 0 |
| TC-098 | `.status v::1` → labeled lines | Explicit 1 |
| TC-108 | `.version.show v::0` → bare semver | Explicit 0 |
| TC-109 | `.version.show v::1` → "Version: X.Y.Z" | Explicit 1 |
| TC-118 | `.version.list v::0` → names only | Explicit 0 |
| TC-119 | `.version.list v::1` → names + descriptions | Explicit 1 |
| TC-164 | `.settings.show v::0` → `key=value` format | Explicit 0 |
| TC-179 | `.settings.get v::0` → bare value only | Explicit 0 |
| TC-245 | Last `v::` wins when duplicated | Duplication |
| TC-428 | `.version.history v::0` → bare version+date | Explicit 0 |
| TC-430 | `.version.history v::2` → full changelog | Explicit 2 |
| EC-1 | Default (absent) resolves to `v::1` | Default Behavior |
| EC-2 | `v::0` consistently minimal across all commands | Cross-Command |
| EC-3 | `v::3` → exit 1, out of range | Invalid: out-of-range |
| EC-4 | `v::-1` → exit 1, out of range | Invalid: negative |
| EC-5 | `v::abc` → exit 1, non-integer | Format Violation |
| EC-6 | `v::` (empty) → exit 1 | Empty Value |
| EC-7 | `v::` accepted by 9 commands, rejected by 3 | Command Scope |
| TC-484 | `verbosity::3` → exit 1 (canonical key, over-range) | Invalid: canonical over-range |
| TC-485 | `verbosity::-1` → exit 1 (canonical key, negative) | Invalid: canonical negative |
| TC-486 | `verbosity::0` accepted via canonical key → exit 0 | Valid: canonical form |

## Test Coverage Summary

- Explicit 0 (minimal output): 5 tests
- Explicit 1 (default/labeled): 2 tests
- Explicit 2 (extended detail): 1 test
- Duplication (last-wins): 1 test
- Default Behavior: 1 test
- Cross-Command: 1 test
- Invalid (out-of-range): 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test
- Canonical form (valid): 1 test
- Canonical form (invalid): 2 tests

**Total:** 20 edge cases

---

### TC-245: Last `v::` wins when duplicated

**Goal:** When `v::` appears multiple times, the last value takes effect (FR-02).
**Setup:** None.
**Command:** `cm .version.list v::0 v::1` (last is v::1)
**Expected Output:** Output shows descriptions (v::1 behavior, not v::0).
**Verification:** output contains description separators.
**Pass Criteria:** Last `v::` value applied.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-1: Default (absent) → `v::1`

**Goal:** Omitting `v::` defaults to labeled output format.
**Setup:** None.
**Command:** `cm .version.list` (no v:: param).
**Expected Output:** Behavior identical to `v::1` (names with descriptions).
**Pass Criteria:** Default equals explicit v::1.
**Source:** [params.md — v:: default: 1](../../params.md)

---

### EC-2: `v::0` consistently minimal across commands

**Goal:** Verbosity 0 always produces the most compact machine-readable output.
**Setup:** None.
**Commands:** Test `v::0` on multiple commands.
**Expected Output:** All produce bare data without labels.
**Pass Criteria:** Consistent minimum-output behavior across commands.
**Source:** [types.md — verbosity levels](../../types.md)

---

### EC-3: `v::3` → exit 1

**Goal:** Value 3 is out of the valid range 0–2.
**Setup:** None.
**Command:** `cm .version.list v::3`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-4: `v::-1` → exit 1

**Goal:** Negative values rejected; v:: is u8 range.
**Setup:** None.
**Command:** `cm .version.list v::-1`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-5: `v::abc` → exit 1

**Goal:** Non-integer strings rejected for Integer parameter.
**Setup:** None.
**Command:** `cm .version.list v::abc`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-6: `v::` (empty) → exit 1

**Goal:** Empty value for required integer is a usage error.
**Setup:** None.
**Command:** `cm .version.list v::`
**Expected Output:** exit code 1; error about v:: requiring a value.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-7: `v::` only for output-formatting commands

**Goal:** Commands without output formatting (`.processes.kill`, `.settings.set`) reject `v::`.
**Setup:** None.
**Command:** `cm .processes.kill v::1`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-484: `verbosity::3` → exit 1 (canonical key over-range)

**Goal:** The canonical `verbosity::` key is range-validated identically to the `v::` alias.
Regression guard for issue-verbosity-bypass: before the fix, `verbosity::3` bypassed the
0–2 range check and silently mapped to level 2 because only `v::` was validated.
**Setup:** None.
**Command:** `cm .version.list verbosity::3`
**Expected Output:** exit code 1; error mentions `verbosity::`.
**Pass Criteria:** Exit 1; no silent level clamping.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-485: `verbosity::-1` → exit 1 (canonical key negative)

**Goal:** Negative value via canonical key is rejected (same constraint as `v::-1`).
**Setup:** None.
**Command:** `cm .version.list verbosity::-1`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-486: `verbosity::0` accepted via canonical key → exit 0

**Goal:** Valid value via canonical `verbosity::` key is accepted, confirming the key
is user-accessible (not purely internal).
**Setup:** None.
**Command:** `cm .version.list verbosity::0`
**Expected Output:** exit code 0; output without "Version:" label (v::0 minimal format).
**Pass Criteria:** Exit 0; no "Version:" prefix in output.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)
