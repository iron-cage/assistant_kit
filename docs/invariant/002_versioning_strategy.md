# Invariant: Versioning Strategy

### Scope

- **Purpose**: Document the shared workspace versioning policy and the allowed divergence mechanism.
- **Responsibility**: State the shared version rule, the rationale for cohesive releases, and when/how per-crate version overrides are permitted.
- **In Scope**: Shared `[workspace.package] version`, cohesive release rationale, per-crate override mechanism.
- **Out of Scope**: Privacy invariant (→ `invariant/001_privacy_invariant.md`), testing strategy (→ `invariant/003_testing_strategy.md`).

### Invariant Statement

All crates share a single version declared in `[workspace.package]` unless explicitly overridden with documented justification.

**Normal state:**
```toml
[workspace.package]
version = "1.0.0"
```

All 10 crates inherit this version via `version.workspace = true` in their `Cargo.toml`.

**Override (exception):** If a crate needs to diverge (e.g., a major API break to only one crate), override with an explicit `version = "x.y.z"` in that crate's `Cargo.toml`. The reason for the divergence must be documented.

### Enforcement Mechanism

The workspace `Cargo.toml` declares the shared version. Crates that need to diverge must explicitly override and cannot silently stay behind.

**Rationale for shared version:** The 10 crates form a cohesive release unit. Changes to session path resolution typically ripple into storage parsing and runner configuration. A shared version prevents consumers from mixing incompatible crate versions from the same workspace.

### Violation Consequences

- Silently diverging versions cause consumer confusion about which combination of crate versions is compatible
- Unintentional version overrides can create publishing inconsistencies

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Crate inventory that shares this version |
| source | `../../Cargo.toml` | workspace.package.version declaration |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Versioning Strategy section |
