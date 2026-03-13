# Test: Output Control Group

Interaction tests for the `v::` (verbosity) and `format::` parameter group.
See [parameter_groups.md](../../parameter_groups.md) and [parameter_interactions.md](../../parameter_interactions.md).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `v::` | u8 (0-2) | 1 | 9 commands (all except `.help`, `.processes.kill`, `.settings.set`) |
| `format::` | text\|json | text | 8 commands (all except `.help`, `.processes.kill`, `.settings.set`, `.version.guard`) |

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-104 | `.status v::0` has ≤ lines than `v::1` | Verbosity ordering |
| TC-245 | Last `v::` wins when duplicated | Duplicate param |
| IT-1 | `v::0 format::json` → json format ignores verbosity | format overrides v:: |
| IT-2 | `v::2 format::json` → same as v::0 format::json | format overrides v:: |
| IT-3 | `v::0` is always machine-parseable across commands | Cross-command v::0 |
| IT-4 | `format::json` always produces valid JSON regardless of v:: | JSON validity |
| IT-5 | `v::1 format::text` → same as default (both explicitly default) | Explicit defaults |
| EC-1 | `v::3 format::json` → exit 1 (v:: range check before format) | Validation order |
| EC-2 | `v::0 format::xml` → exit 1 (format:: check) | Both invalid |
| EC-3 | `v::abc format::json` → exit 1 | v:: type check |
| EC-4 | `v::` and `format::` absent → always text v::1 default | Both absent |

## Test Coverage Summary

- Verbosity ordering: 1 test
- Duplicate param (last-wins): 1 test
- format overrides v:: for JSON: 2 tests
- Cross-command consistency: 2 tests
- Explicit defaults: 1 test
- Validation (invalid v:: + valid format): 1 test
- Validation (valid v:: + invalid format): 1 test
- Validation (invalid v:: type): 1 test
- Both absent (defaults): 1 test

**Total:** 11 interaction tests

---

### IT-1: `v::0 format::json` → JSON ignores verbosity

**Goal:** When `format::json` is specified, verbosity level does not alter the JSON structure.
JSON output is always structured the same regardless of `v::`.
**Setup:** None.
**Command:** `cm .version.list v::0 format::json` vs `cm .version.list v::2 format::json`
**Expected:** Both produce identical JSON output.
**Verification:** stdout of both calls equal.
**Pass Criteria:** JSON output independent of v::.
**Source:** [parameter_interactions.md — format::json overrides v::](../../parameter_interactions.md)

---

### IT-2: `v::2 format::json` identical to `v::0 format::json`

**Goal:** Confirming format independence from verbosity for JSON specifically.
**Setup:** None.
**Command:** Run `.version.list v::2 format::json` and `.version.list v::0 format::json`.
**Expected:** Identical JSON outputs.
**Verification:** outputs equal.
**Pass Criteria:** v:: has no effect on JSON format.

---

### IT-3: `v::0` is machine-parseable across commands

**Goal:** Verbosity 0 always produces the minimal output suitable for scripting.
**Setup:** Appropriate state for each command.
**Commands:** `v::0` on `.status`, `.version.list`, `.settings.get`, `.processes`.
**Expected:** Each produces compact, label-free output.
**Verification:** No label separators in any output.
**Pass Criteria:** Consistent minimum-output behavior.

---

### TC-245: Last `v::` wins on duplicate

**Goal:** FR-02: last occurrence of any duplicate parameter wins.
**Setup:** None.
**Command:** `cm .version.list v::0 v::1`
**Expected:** v::1 behavior (descriptions present) because v::1 is last.
**Verification:** output contains description separators.
**Pass Criteria:** v::1 applied.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-1: `v::3 format::json` → exit 1

**Goal:** Validation catches out-of-range v:: even when format:: is valid.
**Setup:** None.
**Command:** `cm .status v::3 format::json`
**Expected:** Exit 1 (v:: range check fails).
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### EC-2: `v::0 format::xml` → exit 1

**Goal:** Valid v:: does not prevent rejection of invalid format::.
**Setup:** None.
**Command:** `cm .status v::0 format::xml`
**Expected:** Exit 1 (format:: value rejected).
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### EC-4: Both absent → text v::1 defaults

**Goal:** Omitting both parameters applies text format and verbosity 1.
**Setup:** None.
**Command:** `cm .version.list`
**Expected:** Output is labeled text (not JSON, not bare names).
**Verification:** output contains description text; does not start with `[` or `{`.
**Pass Criteria:** Default behavior: labeled text output.
