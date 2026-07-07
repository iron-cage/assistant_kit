# Invariant: Privacy Invariant

### Scope

- **Purpose**: Document the zero-upstream-private-workspace-knowledge constraint that the assistant workspace must always maintain.
- **Responsibility**: State the invariant, enumerate permitted and forbidden dependency types, and explain the rationale.
- **In Scope**: Permitted upstream deps (published companion crates, stdlib), forbidden dep types (private workspace path deps), dependency flow direction.
- **Out of Scope**: Versioning strategy (→ `invariant/002_versioning_strategy.md`), cross-workspace protocol (→ `integration/001_consumer_integration.md`).

### Invariant Statement

This workspace has zero knowledge of any upstream private workspace.

**Permitted upstream dependencies:**
- Published companion crates (`error_tools`, `unilang`, `data_fmt`, `cli_fmt`, …)
- Rust standard library
- Published ecosystem crates (crates.io)

**Forbidden:**
- Path dependencies to any private consumer workspace
- Any type, trait, or concept specific to a consumer workspace's internal job queue, orchestration, or agent layer
- Out-of-workspace path dependencies without a `version` field (makes the depending crate unpublishable)

<!-- BUG-398 task/claude_runner/bug/cancelled/398_bug_txt_external_wplan_artifact_misattributed.md — cited as evidence this forbidden-vocabulary invariant is correctly enforced; no content change required -->

### Enforcement Mechanism

The workspace `Cargo.toml` lists no path deps to any private workspace. Each crate's `Cargo.toml` must not introduce such path deps.

Any out-of-workspace path dep (co-developed crates injected via Docker build contexts or sibling repos) must have a `version` field alongside `path`. Without `version`, Cargo refuses to publish any crate in the dependency chain.

Dependency flow is strictly one-way:
```
published companion crates (error_tools, unilang, …)
  └─ assistant (this workspace)
       └─ consumer workspace (private — depends on assistant via path deps)
```

The consumer workspace depends on assistant; assistant does not depend on the consumer workspace. Allowing the reverse direction would create a circular dependency.

**`missing_inline_in_public_items` boundary:** The workspace lint `missing_inline_in_public_items = "warn"` with `-D warnings` makes missing `#[inline]` a hard error. All public items — including trait impl methods (`fmt`, `source`, `from`, `default`) — require `#[inline]`.

### Violation Consequences

- Adding a consumer workspace path dep creates a circular dependency between workspaces
- Adding consumer workspace types leaks internal orchestration concepts into the published interface
- Any crate depending on a private workspace becomes unpublishable to crates.io

### Features

| File | Relationship |
|------|--------------|
| [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Workspace that this invariant protects |

### Integrations

| File | Relationship |
|------|--------------|
| [integration/001_consumer_integration.md](../integration/001_consumer_integration.md) | Cross-workspace dep protocol that flows in the permitted direction |

### Sources

| File | Relationship |
|------|--------------|
| `../../Cargo.toml` | Workspace dependency declarations |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Privacy Invariant, Dependency Flow sections |
