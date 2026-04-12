# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_assets_core library for consumers building artifact installation tooling.
- **Responsibility**: Index of feature doc instances covering artifact classification and symlink-based install/uninstall semantics.
- **In Scope**: ArtifactKind taxonomy, ArtifactLayout, AssetPaths resolution, install/uninstall idempotency.
- **Out of Scope**: CLI command design (→ `claude_assets/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Artifact Installer](001_artifact_installer.md) | Symlink-based install/uninstall of Claude Code artifacts | ✅ |
