# Pitfall

### Scope

- **Purpose**: Catalog systemic pitfalls — recurring design traps and non-obvious constraints — across `claude_profile`. Each instance documents a class of mistakes, their root causes, and how to avoid them.
- **Responsibility**: Consolidate bug-fix knowledge that reveals structural design constraints not obvious from feature docs alone.
- **In Scope**: Cross-feature pitfall patterns revealed by multiple bugs in the same area.
- **Out of Scope**: Single-occurrence bugs (captured in bug files); feature-specific acceptance criteria (→ `feature/`).

### Overview Table

| ID | Name | Bug refs |
|----|------|---------|
| 001 | [Quota Gate Pitfalls](001_quota_gate_pitfalls.md) | BUG-299, BUG-300, BUG-301 |
| 002 | [Subprocess Integration Pitfalls](002_subprocess_integration_pitfalls.md) | BUG-169, BUG-243, BUG-289, BUG-290 |
| 003 | [Credential Sync Pitfalls](003_credential_sync_pitfalls.md) | BUG-162, BUG-170, BUG-208, BUG-211, BUG-221, BUG-310 |
| 004 | [Account Identity Pitfalls](004_account_identity_pitfalls.md) | BUG-212, BUG-308 |
| 005 | [Ownership Gate Pitfalls](005_ownership_gate_pitfalls.md) | BUG-302, BUG-303, BUG-305, BUG-306 |
| 006 | [Model Override Pitfalls](006_model_override_pitfalls.md) | BUG-300, BUG-311, BUG-312 |
