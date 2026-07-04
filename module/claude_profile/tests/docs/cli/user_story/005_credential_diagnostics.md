# Test: user story 5 — Credential Diagnostics

User acceptance tests for the "Credential Diagnostics" story. Each UA-N case maps to one
Acceptance Criterion from
[docs/cli/user_story/005_credential_diagnostics.md](../../../../docs/cli/user_story/005_credential_diagnostics.md).

**Persona:** Developer troubleshooting authentication failures or verifying account configuration.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `.credentials.status` shows subscription, tier, token validity, and expiry | AC-1 |
| UA-2 | `.token.status` classifies token as Valid / ExpiringSoon / Expired with duration | AC-2 |
| UA-3 | `.paths` resolves all canonical `~/.claude/` file paths | AC-3 |
| UA-4 | `.account.inspect trace::1` shows live endpoint responses and membership selection | AC-4 |

### Test Coverage Summary

- Live credential display: 1 test
- Token classification: 1 test
- Path resolution: 1 test
- Deep diagnostic: 1 test

**Total:** 4 user acceptance tests

---

### UA-1: `.credentials.status` shows subscription, tier, token validity, and expiry — no account store required

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. Credential store may be empty (no named accounts).
- **When:** `clp .credentials.status`
- **Then:** Exit 0. Output includes: subscription type (e.g., `max`), tier (e.g., `default_claude_max_20x`), token status (`valid` / `expiring_soon` / `expired`), and expiry duration (e.g., `~4h`). Succeeds without account store setup — pure read from `~/.claude/.credentials.json`.
- **Exit:** 0
- **Source:** [005_credential_diagnostics.md — AC-1](../../../../docs/cli/user_story/005_credential_diagnostics.md)

---

### UA-2: `.token.status` classifies token as Valid / ExpiringSoon / Expired with exact duration

- **Given (valid):** `~/.claude/.credentials.json` with `expiresAt` = now + 2h.
- **Given (expiring):** `~/.claude/.credentials.json` with `expiresAt` = now + 20min (within default threshold).
- **Given (expired):** `~/.claude/.credentials.json` with `expiresAt` = now - 1h.
- **When:** `clp .token.status` for each case
- **Then (valid):** Exit 0. Status = `Valid`. Duration shown (e.g., `1h 59m`).
- **Then (expiring):** Exit 0. Status = `ExpiringSoon`. Duration shown.
- **Then (expired):** Exit 0. Status = `Expired`. Duration shown as negative or elapsed.
- **Exit:** 0
- **Source:** [005_credential_diagnostics.md — AC-2](../../../../docs/cli/user_story/005_credential_diagnostics.md)

---

### UA-3: `.paths` resolves all canonical `~/.claude/` file paths on the current machine

- **Given:** Standard `~/.claude/` directory exists. `$HOME` is set.
- **When:** `clp .paths`
- **Then:** Exit 0. Output lists all canonical file paths (e.g., credentials file, settings, claude.json). Paths are resolved to absolute paths on the current machine. No environment-specific assumptions embedded in output.
- **Exit:** 0
- **Source:** [005_credential_diagnostics.md — AC-3](../../../../docs/cli/user_story/005_credential_diagnostics.md)

---

### UA-4: `.account.inspect trace::1` shows live endpoint responses and membership selection priority

- **Given:** `alice@acme.com` credentials accessible. All three API endpoints (001, 002, 005) reachable.
- **When:** `clp .account.inspect name::alice@acme.com trace::1`
- **Then:** Exit 0. Diagnostic output includes identity, subscription tier, billing type, and membership list. `trace::1` produces verbose stderr output showing endpoint calls and membership selection priority used to determine the authoritative subscription data.
- **Exit:** 0
- **Source:** [005_credential_diagnostics.md — AC-4](../../../../docs/cli/user_story/005_credential_diagnostics.md)
