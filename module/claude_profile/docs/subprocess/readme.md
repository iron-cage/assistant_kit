# Subprocess Doc Entity

### Scope

- **Purpose**: Document the isolated subprocess layer — the `run_isolated()` contract, credential write-back protocol, and each invocation site within `claude_profile`.
- **Responsibility**: Authoritative reference for how `claude_profile` spawns, monitors, and harvests credentials from isolated Claude binary subprocesses.
- **In Scope**: `run_isolated()` API contract, credential write-back protocol, and the three invocation contexts (token refresh, session touch, browser relogin).
- **Out of Scope**: `run_isolated()` implementation internals in `claude_runner_core`; quota fetch HTTP mechanics; `IsolatedModel` variant definitions (→ `claude_runner_core/src/isolated.rs`).

### Type Declaration

- **Type name**: Subprocess
- **Extends**: Doc Entity (local extension — not a built-in type in `doc_des.rulebook.md`)
- **Instance naming**: `{NNN}_{invocation_context}.md` (NNN = 3-digit ID)
- **Required instance sections**: `### Scope` (4 bullets), invocation-specific content sections
- **Optional instance sections**: Typed reference sections (`### Features`, `### Algorithms`, `### State Machines`, `### Invariants`)

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining subprocess instances | ✅ |
| 001 | [`run_isolated()` Contract](001_run_isolated_contract.md) | API signature, isolation mechanism, result types, error types | ✅ |
| 002 | [Credential Write-Back Protocol](002_credential_writeback.md) | How refreshed credentials flow from subprocess back to disk and live session | ✅ |
| 003 | [Token Refresh Invocation](003_token_refresh_invocation.md) | When and how `refresh_account_token()` is called for expired token recovery | ✅ |
| 004 | [Session Touch Invocation](004_session_touch_invocation.md) | When and how `refresh_account_token()` is called for idle window activation | ✅ |
| 005 | [Browser Relogin Invocation](005_relogin_invocation.md) | When and how `claude` is spawned with inherited TTY for RT-expired recovery | ✅ |
