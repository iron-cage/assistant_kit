# Test: user story 4 — Scripted Pipeline Automation

User acceptance tests for the "Scripted Pipeline Automation" story. Each UA-N case maps
to one Acceptance Criterion from
[docs/cli/user_story/004_scripted_automation.md](../../../../docs/cli/user_story/004_scripted_automation.md).

**Persona:** DevOps engineer integrating `clp` into CI/CD pipelines and shell scripts.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `format::json` on any format-capable command returns valid JSON | AC-1 |
| UA-2 | `get::FIELD` returns a single bare scalar value with no headers | AC-2 |
| UA-3 | Exit codes are deterministic and match documented triggers | AC-3 |
| UA-4 | `only_next::1 get::account` returns the recommended account as a bare string | AC-4 |

### Test Coverage Summary

- JSON output: 1 test
- Scalar extraction: 1 test
- Exit code determinism: 1 test
- Pipeline extraction: 1 test

**Total:** 4 user acceptance tests

---

### UA-1: `format::json` on any format-capable command returns valid JSON

- **Given:** Active credentials. At least one saved account.
- **When (a):** `clp .token.status format::json`
- **When (b):** `clp .accounts format::json`
- **When (c):** `clp .credentials.status format::json`
- **Then (a):** Exit 0. Output is valid JSON object parseable by `jq .`.
- **Then (b):** Exit 0. Output is valid JSON array parseable by `jq .`.
- **Then (c):** Exit 0. Output is valid JSON object parseable by `jq .`.
- **Exit:** 0
- **Source:** [004_scripted_automation.md — AC-1](../../../../docs/cli/user_story/004_scripted_automation.md)

---

### UA-2: `get::FIELD` returns a single bare scalar value with no headers

- **Given:** Active credentials with a known subscription type (e.g., `sub = "max"`).
- **When:** `clp .credentials.status get::subscription`
- **Then:** Exit 0. stdout is a single line containing exactly the subscription value (e.g., `max`) with no JSON wrapper, no headers, no padding. Suitable for direct shell variable capture: `RESULT=$(clp .credentials.status get::subscription)`.
- **Exit:** 0
- **Source:** [004_scripted_automation.md — AC-2](../../../../docs/cli/user_story/004_scripted_automation.md)

---

### UA-3: Exit codes are deterministic and match documented triggers

- **Given:** Credential store configured with `alice@acme.com` saved.
- **When (a):** `clp .token.status` with valid credentials → expect exit 0
- **When (b):** `clp .account.use name::nobody@acme.com` → account not found → expect exit 2
- **When (c):** `clp .account.save name::notanemail` → invalid format → expect exit 1
- **Then:** Each command exits with exactly the documented code. Scripts that branch on `$?` receive consistent, predictable values across invocations and platforms.
- **Exit:** 0 (a), 2 (b), 1 (c)
- **Source:** [004_scripted_automation.md — AC-3](../../../../docs/cli/user_story/004_scripted_automation.md)

---

### UA-4: `only_next::1 get::account` returns the recommended account as a bare string

- **Given:** Multiple saved accounts with differing quota. A `next::` strategy is configured (or defaults to endurance).
- **When:** `clp .usage only_next::1 get::account`
- **Then:** Exit 0. stdout is a single line containing exactly the email address of the recommended next account (e.g., `alice@acme.com`). No table headers, no other rows. Suitable for: `NEXT=$(clp .usage only_next::1 get::account) && clp .account.use name::$NEXT`.
- **Exit:** 0
- **Source:** [004_scripted_automation.md — AC-4](../../../../docs/cli/user_story/004_scripted_automation.md)
