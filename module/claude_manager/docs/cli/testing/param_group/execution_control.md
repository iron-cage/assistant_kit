# Test: Execution Control Group

Interaction tests for the `dry::`, `force::`, and `interval::` parameter group.
See [parameter_groups.md](../../parameter_groups.md) and [parameter_interactions.md](../../parameter_interactions.md).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `dry::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set` |
| `force::` | Boolean (0/1) | 0 | `.version.install`, `.version.guard`, `.processes.kill` |
| `interval::` | u64 | 0 | `.version.guard` only |

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `dry::1` always wins over `force::1` | dry+force precedence |
| IT-2 | `dry::0 force::1` → force active (dry::0 means off) | Explicit false |
| IT-3 | `dry::1 force::0` → dry active, force off | Explicit false |
| IT-4 | `dry::1 interval::0` → one-shot dry-run | dry+interval |
| IT-5 | `interval::N` (N>0) starts watch loop | interval>0 |
| IT-6 | `force::1` bypasses idempotency on `.version.guard` | force alone |
| IT-7 | `force::1` on `.processes.kill` → SIGKILL directly | force SIGKILL |
| EC-1 | `dry::1 force::1 interval::0` → dry wins, one-shot | All three |
| EC-2 | `interval::5 dry::1` → watch loop, but each iteration is dry-run | watch+dry |
| EC-3 | `force::1` without `dry::1` → real operation | force alone |
| EC-4 | `dry::0 force::0` explicit → same as both absent | Explicit off |

## Test Coverage Summary

- dry+force precedence: 3 tests (IT-1, IT-2, IT-3)
- dry+interval: 2 tests (IT-4, EC-2)
- interval watch mode: 1 test (IT-5)
- force alone: 2 tests (IT-6, IT-7, EC-3)
- All three combined: 1 test (EC-1)
- Explicit off: 1 test (EC-4)

**Total:** 11 interaction tests

---

### IT-1: `dry::1` wins over `force::1`

**Goal:** `dry::` is the stronger constraint; `force::` is ignored when `dry::1` is set.
**Setup:** None.
**Commands:**
- `cm .version.install dry::1 force::1`
- `cm .version.guard dry::1 force::1`
- `cm .processes.kill dry::1 force::1`
**Expected:** All exit 0 with `[dry-run]` prefix; no real action performed.
**Verification:** `[dry-run]` present in output; no side effects.
**Pass Criteria:** Dry-run wins on all three applicable commands.
**Source:** [parameter_interactions.md — dry+force](../../parameter_interactions.md)

---

### IT-2: `dry::0 force::1` → force active

**Goal:** Explicit `dry::0` disables dry-run; `force::1` takes effect.
**Setup:** Preference stored; version matches.
**Command:** `cm .version.guard dry::0 force::1`
**Expected:** Real install triggered (bypasses match check).
**Verification:** No `[dry-run]` in output; install log present.
**Pass Criteria:** Force behavior active; dry-run off.
**Source:** [parameter_interactions.md](../../parameter_interactions.md)

---

### IT-4: `dry::1 interval::0` → one-shot dry-run

**Goal:** One-shot mode with dry-run: single check, preview only, immediate exit.
**Setup:** No preference stored.
**Command:** `cm .version.guard dry::1 interval::0`
**Expected:** Exit 0; `[dry-run]` present; process exits immediately.
**Verification:** `[dry-run]` in output; no loop.
**Pass Criteria:** Exit 0; one-shot; no side effects.
**Source:** [params.md — interval::0](../../params.md)

---

### IT-5: `interval::N` starts watch loop

**Goal:** Non-zero interval produces a watch process that loops.
**Setup:** No preference stored.
**Command:** `timeout 3 cm .version.guard interval::5`
**Expected:** At least one status line emitted; process kept alive by timeout.
**Verification:** Process does not exit within 3 seconds on its own.
**Pass Criteria:** Watch loop active; terminated by `timeout`.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)

---

### EC-1: `dry::1 force::1 interval::0` → all three together

**Goal:** All three execution control params in combination behave predictably.
**Setup:** No preference stored.
**Command:** `cm .version.guard dry::1 force::1 interval::0`
**Expected:** Exit 0; `[dry-run]` in output; one-shot; no install.
**Verification:** `[dry-run]` present; exits immediately; no side effects.
**Pass Criteria:** dry wins; one-shot mode; no action.
**Source:** [parameter_interactions.md](../../parameter_interactions.md)

---

### EC-2: `interval::5 dry::1` → watch loop with dry-run

**Goal:** Watch mode and dry-run combine: each loop iteration shows preview, no install.
**Setup:** No preference stored.
**Command:** `timeout 6 cm .version.guard interval::5 dry::1`
**Expected:** At least one `[dry-run]` line in output; process loops.
**Verification:** `[dry-run]` in output; process does not exit in 5 seconds.
**Pass Criteria:** Both watch and dry-run active.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)
