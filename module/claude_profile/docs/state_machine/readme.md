# State Machine

### Scope

- **Purpose**: Document the lifecycle state machines for key domain concepts in `claude_profile` — accounts, tokens, quota windows, ownership, and quota measurement.
- **Responsibility**: Each instance defines states, transitions, and the operations that drive them. Feature docs contain requirements; state machine docs contain structural lifecycle models.
- **In Scope**: State definitions, valid transitions, terminal states, and invariants for each lifecycle.
- **Out of Scope**: Full feature acceptance criteria (→ `feature/`); algorithm internals (→ `algorithm/`).

### Overview Table

| ID | Name | Domain |
|----|------|--------|
| 001 | [Account Lifecycle](001_account_lifecycle.md) | Credential store presence and active state |
| 002 | [OAuth Token Lifecycle](002_oauth_token_lifecycle.md) | `accessToken`/`refreshToken` validity states |
| 003 | [Session Window Lifecycle](003_session_window_lifecycle.md) | 5h/7d/7d-Sonnet quota timer active/idle states |
| 004 | [Ownership Lifecycle](004_ownership_lifecycle.md) | Account `owner` field claimed/unclaimed/foreign states |
| 005 | [Quota Measurement Lifecycle](005_quota_measurement_lifecycle.md) | History ring buffer fill and approximation readiness |
