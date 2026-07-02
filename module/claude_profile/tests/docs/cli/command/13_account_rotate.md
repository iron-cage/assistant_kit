# Command :: 13. `.account.rotate` — Integration Tests

> **DEPRECATED** — `.account.rotate` is now a redirector (Feature 016). Always exits 1 with migration message. Rotation moved to `.usage rotate::1` (Feature 038). Test cases below verify redirector behavior only.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Always exits 1 | Any invocation of `.account.rotate` | 1 |
| IT-2 | Error message references `.usage rotate` | No args; stderr/stdout contains `.usage rotate` | 1 |
| IT-3 | No mutation on exit 1 | Two accounts present; `_active` unchanged after deprecated call | 1 |

---

### IT-1: Always exits 1

- **Given:** Any credential store state (zero or more accounts)
- **When:** `clp .account.rotate`
- **Then:** Exits 1. Command is DEPRECATED — always redirects with exit code 1.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md)

---

### IT-2: Error message references `.usage rotate`

- **Given:** Any credential store state
- **When:** `clp .account.rotate` (no arguments)
- **Then:** Exits 1. stderr or stdout contains the string `.usage rotate`.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md), [feature/038_usage_strategy_rotate.md](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-3: No mutation on exit 1

- **Given:** Two accounts saved; one is active (has `_active` per-machine marker set)
- **When:** `clp .account.rotate`
- **Then:** Exits 1. The `_active` marker is unchanged — no account file was mutated.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md)
