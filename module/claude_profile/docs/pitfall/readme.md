# Pitfall Doc Entity

### Scope

- **Purpose**: Catalog systemic pitfalls — recurring design traps and non-obvious constraints — across `claude_profile`. Each instance documents a class of mistakes, their root causes, and how to avoid them.
- **Responsibility**: Consolidate bug-fix knowledge that reveals structural design constraints not obvious from feature docs alone.
- **In Scope**: Cross-feature pitfall patterns revealed by multiple bugs in the same area.
- **Out of Scope**: Single-occurrence bugs (captured in bug files); feature-specific acceptance criteria (→ `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining pitfall instances | ✅ |
| 001 | [Quota Gate Pitfalls](001_quota_gate_pitfalls.md) | Quota gate ordering and cross-feature visibility constraints | ✅ |
| 002 | [Subprocess Integration Pitfalls](002_subprocess_integration_pitfalls.md) | Subprocess invocation ordering and write-back race conditions | ✅ |
| 003 | [Credential Sync Pitfalls](003_credential_sync_pitfalls.md) | Credential file sync timing and write-back protocol pitfalls | ✅ |
| 004 | [Account Identity Pitfalls](004_account_identity_pitfalls.md) | Account identity resolution and marker matching pitfalls | ✅ |
| 005 | [Ownership Gate Pitfalls](005_ownership_gate_pitfalls.md) | Ownership gate enforcement edge cases and bypass scenarios | ✅ |
| 006 | [Model Override Pitfalls](006_model_override_pitfalls.md) | Model override ordering and settings persistence pitfalls | ✅ |
| 007 | [Label Selection Branch-Priority Pitfalls](007_label_selection_branch_priority_pitfalls.md) | Branch-priority label functions silently mask one true condition behind another checked earlier | ✅ |
