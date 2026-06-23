# 01_workspace_design

Test spec for `docs/feature/001_workspace_design.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| WD-1 | Workspace member completeness | ⏳ |

## Cases

### WD-1: All documented workspace members present in Cargo.toml

- **Given:** The workspace `Cargo.toml` at the repository root
- **When:** The `[workspace.members]` list is parsed
- **Then:** All 17 crates from the documented crate inventory are present: `claude_storage_core`, `claude_auth`, `claude_quota`, `claude_core`, `claude_profile_core`, `claude_version_core`, `claude_runner_core`, `claude_assets_core`, `claude_profile`, `claude_storage`, `claude_runner`, `dream`, `claude_version`, `claude_assets`, `assistant`, `assistant_kit`, `runbox`
