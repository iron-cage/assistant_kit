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

Most crates inherit this version via `version.workspace = true` in their `Cargo.toml`. The workspace currently has 16 publishable crates; 11 follow the shared version and 5 use explicit overrides (see table below).

**Override (exception):** If a crate needs to diverge (e.g., a major API break to only one crate, or a standalone primitive with its own release cadence), override with an explicit `version = "x.y.z"` in that crate's `Cargo.toml`. The reason for the divergence must be documented.

**Current version divergences:**

| Crate | Version | Rationale |
|-------|---------|-----------|
| `claude_auth` | `0.1.0` | Standalone Layer * primitive; early-stage release; own cadence |
| `claude_quota` | `0.1.0` | Standalone Layer * primitive; early-stage release; own cadence |
| `dream` | `1.2.0` | Library facade; explicitly versioned to track its own API additions independently of workspace cadence |
| `assistant` | `1.2.0` | Super-app binary; version tracks `dream` |
| `assistant_kit` | `0.1.0` | Layer 3 library facade; early-stage release; own cadence |

### Enforcement Mechanism

The workspace `Cargo.toml` declares the shared version. Crates that need to diverge must explicitly override and cannot silently stay behind.

**Rationale for shared version:** The 11 cohesive crates form a coherent release unit. Changes to session path resolution typically ripple into storage parsing and runner configuration. A shared version prevents consumers from mixing incompatible crate versions from the same workspace.

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
