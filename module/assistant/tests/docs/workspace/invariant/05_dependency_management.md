# 05_dependency_management

Test spec for `docs/invariant/005_dependency_management.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| DM-1 | Dependency centralization | ⏳ |
| DM-2 | Workspace reference usage | ⏳ |

## Cases

### DM-1: No bare version declarations in crate Cargo.toml files

- **Given:** All crate-level `Cargo.toml` files (under `module/` and `contract/`)
- **When:** Dependency sections (`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`) are inspected
- **Then:** No external dependency uses a bare `dep_name = "x.y.z"` declaration; all external deps reference the workspace (`{ workspace = true }`)

### DM-2: All external deps use workspace = true

- **Given:** All crate-level `Cargo.toml` dependency entries
- **When:** Each external dependency (non-path, non-workspace-member) is inspected
- **Then:** Every entry includes `workspace = true`; no crate-local version override for shared external deps
