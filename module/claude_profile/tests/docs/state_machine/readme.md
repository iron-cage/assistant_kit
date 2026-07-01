# State Machine Test Cases

Test-lens documentation for `docs/state_machine/`. Each spec file covers one state machine
and documents AC-N cases for lifecycle transition and gate enforcement verification. Tests
live in `tests/` of the crate that owns the state machine logic.

### Scope

- **Purpose**: AC-N test cases for lifecycle state machines in `claude_profile`.
- **Responsibility**: Index of per-state-machine correctness spec files; case prefix `AC-`.
- **In Scope**: State machines from `docs/state_machine/` that have been validated or are
  under active work.
- **Out of Scope**: Feature behavioral tests (→ `feature/`); algorithm internals (→ `algorithm/`).

### Overview Table

| Spec | State Machine | Status |
|------|---------------|--------|
| 001_account_lifecycle.md | AC spec for SM-001 — Account Lifecycle (absent/saved/active) | ✅ |
| 002_oauth_token_lifecycle.md | AC spec for SM-002 — OAuth Token Lifecycle (valid/at_expired/rt_expired) | ✅ |
| 003_session_window_lifecycle.md | AC spec for SM-003 — Session Window Lifecycle (idle/active/exhausted) | ✅ |
| 004_ownership_lifecycle.md | AC spec for SM-004 — Ownership Lifecycle (unclaimed/owned_here/owned_elsewhere) | ✅ |
| 005_quota_measurement_lifecycle.md | AC spec for SM-005 — Quota Measurement Lifecycle (empty/single/linear/quadratic/full) | ✅ |
