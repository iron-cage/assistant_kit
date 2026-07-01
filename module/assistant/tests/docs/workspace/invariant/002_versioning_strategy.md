# 02_versioning_strategy

Test spec for `docs/invariant/002_versioning_strategy.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| VS-1 | Shared version declared | ✅ |
| VS-2 | Non-override crates inherit workspace version | ✅ |

## Cases

### VS-1: Workspace declares shared version in workspace.package

- **Given:** The workspace `Cargo.toml` at the repository root
- **When:** The `[workspace.package]` section is parsed
- **Then:** A `version` field is present with a valid semver value

### VS-2: Non-override crates use version.workspace = true

- **Given:** All crate-level `Cargo.toml` files; the 5 documented override crates (`claude_auth`, `claude_quota`, `dream`, `assistant`, `assistant_kit`)
- **When:** Each crate's `[package]` section is inspected
- **Then:** Crates NOT in the override list have `version.workspace = true`; crates IN the override list have an explicit `version = "x.y.z"` that differs from the workspace version
