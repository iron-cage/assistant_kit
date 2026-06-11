# Test: verb::status

Behavioral contract tests for the `status` verb. Verifies full idempotency, read-only
behavior, and pre-condition enforcement for both nouns (`token` and `credentials`) as defined in
[docs/cli/command_verb/010_status.md](../../../../docs/cli/command_verb/010_status.md).

**Idempotency:** Yes — both commands are pure reads; repeated calls return the same result for the same credential state.
**State Pattern:** Reads state — no local files written for either noun.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | `.token.status` called twice returns same classification | Idempotency |
| BV-2 | `.token.status` read is purely non-mutating | State Transition |
| BV-3 | `.token.status` with absent credentials file exits 2 | Pre-condition |
| BV-4 | `.credentials.status` called twice returns same output | Idempotency |

### Test Coverage Summary

- Idempotency: 2 tests (one per noun)
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 4 behavioral contract tests

---

### BV-1: `.token.status` called twice returns same classification

- **Given:** `~/.claude/.credentials.json` exists with `expiresAt` field set to a time in the future (valid token). No time passes between calls.
- **When:** `clp .token.status` called twice in immediate succession
- **Then:** Both calls exit 0. Both calls return the same classification (`Valid` or `ExpiringSoon`). No files modified.
- **Exit:** 0
- **Source:** [010_status.md — Idempotency](../../../../docs/cli/command_verb/010_status.md#idempotency)

---

### BV-2: `.token.status` read is purely non-mutating

- **Given:** `~/.claude/.credentials.json` exists with parseable `expiresAt`. Record mtime of `~/.claude/.credentials.json`.
- **When:** `clp .token.status`
- **Then:** Exit 0. mtime of `~/.claude/.credentials.json` unchanged. No new files created. Token classification reported on stdout.
- **Exit:** 0
- **Source:** [010_status.md — State Transition Pattern](../../../../docs/cli/command_verb/010_status.md#state-transition-pattern)

---

### BV-3: `.token.status` with absent credentials file exits 2

- **Given:** `~/.claude/.credentials.json` does NOT exist.
- **When:** `clp .token.status`
- **Then:** Exit 2. Error message on stderr referencing absent or unreadable credentials file.
- **Exit:** 2
- **Source:** [010_status.md — Behavioral Contract](../../../../docs/cli/command_verb/010_status.md#behavioral-contract)

---

### BV-4: `.credentials.status` called twice returns same output

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. Active account marker set. No changes to credential files between calls.
- **When:** `clp .credentials.status` called twice in immediate succession
- **Then:** Both calls exit 0. Both calls produce identical stdout output. No files written or modified.
- **Exit:** 0
- **Source:** [010_status.md — Idempotency](../../../../docs/cli/command_verb/010_status.md#idempotency)
