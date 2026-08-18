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

**Status note (2026-08-18):** stays ⏳ — the Then-clause literally requires `cargo package --allow-dirty` exit 0, which currently fails at dependency resolution (exit 101, "no matching package named `claude_journal_viewer`" — unpublished dependency, user-gated; see `task/assistant/verified/003_fix_build_rs_publish_panic.md` Blocked Reason). The build.rs mechanism under test IS verified: a faithful sandbox simulation (real `build.rs` verbatim, `CARGO_FEATURE_ENABLED` set, all sibling dirs absent) builds exit 0 with zero panics and generates the empty fallback registry (BUG-003 Verification Record, logs `-0116`/`-0117`). Flip to ✅ only when the literal When/Then commands pass end-to-end.
