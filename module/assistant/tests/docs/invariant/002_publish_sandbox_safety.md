# 02_publish_sandbox_safety

Test spec for `docs/invariant/002_publish_sandbox_safety.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| PS-1 | Publish sandbox safety | ⏳ |

## Cases

### PS-1: build.rs compiles and exits 0 in cargo publish sandbox

- **Given:** The `assistant` crate source tree only (no sibling crates present at `../claude_runner/`, `../claude_assets/`, `../claude_storage/`)
- **When:** `cargo package --allow-dirty` is run from the `module/assistant/` directory
- **Then:** Exits 0; `build.rs` completes without panic; stderr contains no `Failed to read` message
