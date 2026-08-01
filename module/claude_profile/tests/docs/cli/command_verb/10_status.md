# Test: verb::status

Behavioral contract tests for the `status` verb. Verifies idempotency and read-only
behavior for the `credentials` noun — the `token` noun's `.token.status` command has
been removed (BV-1 through BV-3 below) — as defined in
[docs/cli/command_verb/010_status.md](../../../../docs/cli/command_verb/010_status.md).

**Idempotency:** Yes — `.credentials.status` is a pure read; repeated calls return the same result for the same credential state.
**State Pattern:** Reads state — no local files written.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | N/A — `.token.status` removed | Idempotency |
| BV-2 | N/A — `.token.status` removed | State Transition |
| BV-3 | N/A — `.token.status` removed | Pre-condition |
| BV-4 | `.credentials.status` called twice returns same output | Idempotency |

### Test Coverage Summary

- Idempotency: 1 test (BV-4; BV-1 superseded)
- State Transition: 0 tests (BV-2 superseded)
- Pre-condition: 0 tests (BV-3 superseded)

**Total:** 1 behavioral contract test (3 superseded — see per-case notes)

---

### BV-1: N/A — `.token.status` removed

> **N/A** — This case verified `.token.status` called twice returned the same classification. `.token.status` no longer exists; BV-4 covers the same idempotency guarantee for the surviving `.credentials.status` command with a strictly broader assertion (identical full output, which includes the `Token:`/`Expires:` classification).
> Becomes testable when: no committed task.

---

### BV-2: N/A — `.token.status` removed

> **N/A** — This case verified `.token.status` was purely non-mutating (mtime unchanged, no new files). `.credentials.status` carries the same read-only guarantee — documented in `docs/cli/command_verb/010_status.md` Post-conditions ("No files written or modified") and exercised indirectly by `tests/docs/cli/command/10_credentials_status.md` IT-8 (stable output across repeated invocations). No dedicated mtime-check case exists under the surviving command; the property is covered structurally rather than by a standalone test.
> Becomes testable when: no committed task.

---

### BV-3: N/A — `.token.status` removed

> **N/A** — This case verified a missing `~/.claude/.credentials.json` exited 2 under `.token.status`. Superseded by `tests/docs/cli/command/10_credentials_status.md` IT-4 (identical file-absence contract for the surviving `.credentials.status` command).
> Becomes testable when: no committed task.

---

### BV-4: `.credentials.status` called twice returns same output

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. Active account marker set. No changes to credential files between calls.
- **When:** `clp .credentials.status` called twice in immediate succession
- **Then:** Both calls exit 0. Both calls produce identical stdout output. No files written or modified.
- **Exit:** 0
- **Source:** [010_status.md — Idempotency](../../../../docs/cli/command_verb/010_status.md#idempotency)
