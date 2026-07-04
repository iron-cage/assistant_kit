# State Machine Doc Entity

### Scope

- **Purpose**: Document the lifecycle state machines for key domain concepts in `claude_profile` — accounts, tokens, quota windows, ownership, and quota measurement.
- **Responsibility**: Each instance defines states, transitions, and the operations that drive them. Feature docs contain requirements; state machine docs contain structural lifecycle models.
- **In Scope**: State definitions, valid transitions, terminal states, and invariants for each lifecycle.
- **Out of Scope**: Full feature acceptance criteria (→ `feature/`); algorithm internals (→ `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining state machine instances | ✅ |
| 001 | [Account Lifecycle](001_account_lifecycle.md) | Lifecycle states for credential store presence and active account marker | ✅ |
| 002 | [OAuth Token Lifecycle](002_oauth_token_lifecycle.md) | Validity state transitions for OAuth access and refresh tokens | ✅ |
| 003 | [Session Window Lifecycle](003_session_window_lifecycle.md) | Active/idle transitions for 5h, 7d, and 7d-Sonnet quota windows | ✅ |
| 004 | [Ownership Lifecycle](004_ownership_lifecycle.md) | Claimed/unclaimed/foreign transitions for account `owner` field | ✅ |
| 005 | [Quota Measurement Lifecycle](005_quota_measurement_lifecycle.md) | Fill and approximation readiness states for quota measurement ring buffer | ✅ |
