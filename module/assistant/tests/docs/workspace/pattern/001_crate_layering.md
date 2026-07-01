# 01_crate_layering

Test spec for `docs/pattern/001_crate_layering.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| CL-1 | No same-layer dependencies | ✅ |
| CL-2 | Downward-only dependency flow | ✅ |

## Cases

### CL-1: No Layer N crate depends on another Layer N crate

- **Given:** The documented layer assignments: Layer 0 (`claude_core`), Layer 1 (`claude_assets_core`, `claude_profile_core`, `claude_version_core`, `claude_runner_core`), Layer 2 (`dream`, `claude_assets`, `claude_version`, `claude_runner`, `claude_profile`, `claude_storage`), Layer 3 (`assistant`, `assistant_kit`), Layer * (`claude_storage_core`, `claude_auth`, `claude_quota`, `runbox`)
- **When:** Each crate's `[dependencies]` section is parsed and cross-referenced against layer assignments
- **Then:** No crate lists a dependency on another crate assigned to the same layer; Layer * crates have zero workspace dependencies

### CL-2: Dependencies flow downward only

- **Given:** The same layer assignments as CL-1
- **When:** Each crate's workspace dependencies are resolved to their layer assignments
- **Then:** Every workspace dependency targets a strictly lower layer number; no upward dependency (Layer 1 → Layer 2 or Layer 2 → Layer 3); Layer * crates are excluded from this check (they have no workspace deps)
