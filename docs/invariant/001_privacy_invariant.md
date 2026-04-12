# Invariant: Privacy Invariant

### Scope

- **Purpose**: Document the zero-upstream-private-workspace-knowledge constraint that the agent_kit workspace must always maintain.
- **Responsibility**: State the invariant, enumerate permitted and forbidden dependency types, and explain the rationale.
- **In Scope**: Permitted upstream deps (published wtools, stdlib), forbidden dep types (private workspace path deps), dependency flow direction.
- **Out of Scope**: Versioning strategy (→ `invariant/002_versioning_strategy.md`), cross-workspace protocol (→ `integration/001_consumer_integration.md`).

### Invariant Statement

This workspace has zero knowledge of any upstream private workspace.

**Permitted upstream dependencies:**
- Published wtools crates (error_tools, unilang, former, …)
- Rust standard library
- Published ecosystem crates (crates.io)

**Forbidden:**
- Path dependencies to any private consumer workspace
- Any type, trait, or concept specific to a consumer workspace's internal job queue, orchestration, or agent layer

### Enforcement Mechanism

The workspace `Cargo.toml` lists no path deps to any private workspace. Each crate's `Cargo.toml` must not introduce such path deps.

Dependency flow is strictly one-way:
```
wtools (published crates)
  └─ agent_kit (this workspace)
       └─ consumer workspace (private — depends on agent_kit via path deps)
```

The consumer workspace depends on agent_kit; agent_kit does not depend on the consumer workspace. Allowing the reverse direction would create a circular dependency.

**`missing_inline_in_public_items` boundary:** The workspace lint `missing_inline_in_public_items = "warn"` with `-D warnings` makes missing `#[inline]` a hard error. All public items — including trait impl methods (`fmt`, `source`, `from`, `default`) — require `#[inline]`.

### Violation Consequences

- Adding a consumer workspace path dep creates a circular dependency between workspaces
- Adding consumer workspace types leaks internal orchestration concepts into the published interface
- Any crate depending on a private workspace becomes unpublishable to crates.io

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Workspace that this invariant protects |
| integration | [integration/001_consumer_integration.md](../integration/001_consumer_integration.md) | Cross-workspace dep protocol that flows in the permitted direction |
| source | `../../Cargo.toml` | Workspace dependency declarations |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Privacy Invariant, Dependency Flow sections |
