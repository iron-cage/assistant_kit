# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints of the assistant aggregation layer that must never be violated.
- **Responsibility**: Index of invariant doc instances covering Layer 2 aggregation completeness.
- **In Scope**: Registration completeness requirements for Layer 2 crates in the `ast` registry.
- **Out of Scope**: Feature design (→ `feature/`), individual Layer 2 crate constraints (→ each crate's `docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Aggregation Completeness](001_aggregation_completeness.md) | Layer 2 register_commands() and COMMANDS_YAML contract enforcement | ✅ |
| 002 | [Publish Sandbox Safety](002_publish_sandbox_safety.md) | build.rs must not panic during cargo publish verification | ⏳ |
