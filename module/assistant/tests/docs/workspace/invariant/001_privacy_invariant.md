# 01_privacy_invariant

Test spec for `docs/invariant/001_privacy_invariant.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| PI-1 | No private workspace path deps | ✅ |
| PI-2 | Out-of-workspace path deps carry version | ✅ |

## Cases

### PI-1: No private workspace path dependencies

- **Given:** All `Cargo.toml` files across the workspace (root + crate-level)
- **When:** All `path =` dependency values are inspected
- **Then:** No path dependency points to a private consumer workspace directory; all `path` values resolve within the workspace or to published companion crate sibling directories

### PI-2: All out-of-workspace path deps include a version field

- **Given:** The workspace `Cargo.toml` `[workspace.dependencies]` section
- **When:** Entries with `path = "../.."` (out-of-workspace) are inspected
- **Then:** Every such entry also has a `version` field; a path-only entry without version is a violation (makes the crate unpublishable)
