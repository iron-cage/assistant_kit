# 01_super_app_aggregation

### Scope

- **Purpose**: FT- test cases verifying Layer 2 command registration, first-wins precedence, and static YAML aggregation in the `ast` binary.
- **Responsibility**: Acceptance criteria confirming every Layer 2 crate is reachable through the aggregated registry with correct precedence and routing.
- **In Scope**: `build_registry()` completeness across all five Layer 2 crates, command-name collision precedence, static YAML PHF dispatch, `.claude` stub redirect behavior.
- **Out of Scope**: Aggregation completeness invariant (-> `../../../docs/invariant/001_aggregation_completeness.md`), publish sandbox safety invariant (-> `../../../docs/invariant/002_publish_sandbox_safety.md`).

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| FT-1 | L2 registration completeness | ✅ |
| FT-2 | First-wins precedence | ✅ |
| FT-3 | Static YAML command registration | ✅ |
| FT-4 | .claude stub routing | ✅ |

## Cases

### FT-1: All five Layer 2 crates contribute commands to ast

- **Given:** Default `ast` binary built with `--features enabled`
- **When:** A representative command from each Layer 2 crate is invoked: `.kinds` (assets), `.version.show` (version), `.paths` (profile), `.claude` (runner), `.show` (storage)
- **Then:** Each command is found in the registry and dispatched; exit code is 0 or 2 (runtime error acceptable for missing data), never 1 (unknown command)

### FT-2: First-wins precedence for overlapping command names

- **Given:** `claude_version` and `claude_storage` both define `.status`; `claude_version` is registered before `claude_storage` in `build_registry()`
- **When:** `ast .status` is invoked in a temp HOME
- **Then:** The command resolves to `claude_version`'s system status implementation (not storage status); exit code 0

### FT-3: Static YAML commands reachable through PHF map

- **Given:** `build.rs` aggregated YAML from `claude_runner` and `claude_storage` into `static_commands.rs`
- **When:** YAML-backed commands are invoked: `ast .show`, `ast .count`, `ast .search`, `ast .export`
- **Then:** Each command is dispatched via the `register_static_commands()` PHF routine map; exit code 0 or 2 (missing data acceptable)

### FT-4: .claude stub prints redirect instead of executing

- **Given:** `ast` binary (not `clr`)
- **When:** `ast .claude` is invoked
- **Then:** stdout contains "For Claude Code execution, use clr directly"; exit code 0; no subprocess spawned
