# Subprocess Test Cases

Test-lens documentation for `docs/subprocess/`. Each spec file covers one subprocess
contract and documents AC-N cases for invocation correctness verification. Tests live
in `tests/` of the crate that owns the subprocess logic.

### Scope

- **Purpose**: AC-N test cases for subprocess invocation contracts in `claude_profile`.
- **Responsibility**: Index of per-subprocess correctness spec files; case prefix `AC-`.
- **In Scope**: Subprocess docs from `docs/subprocess/` that have been validated or are
  under active work.
- **Out of Scope**: Feature behavioral tests (→ `feature/`); algorithm internals (→ `algorithm/`).

### Overview Table

| Spec | Subprocess | Status |
|------|-----------|--------|
| 001_run_isolated_contract.md | AC spec for Subprocess-001 — run_isolated Contract (sole-caller, args, expiresAt=1) | ✅ |
| 002_credential_writeback.md | AC spec for Subprocess-002 — Credential Write-Back Protocol (live-safe, RT-rotation) | ✅ |
| 003_token_refresh_invocation.md | AC spec for Subprocess-003 — Token Refresh Invocation (should_refresh predicate) | ✅ |
| 004_session_touch_invocation.md | AC spec for Subprocess-004 — Session Touch Invocation (apply_touch predicate, gates) | ✅ |
| 005_relogin_invocation.md | AC spec for Subprocess-005 — Browser Relogin Invocation (TTY, name resolution) | ✅ |
