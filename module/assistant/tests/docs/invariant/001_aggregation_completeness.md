# 01_aggregation_completeness

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| IC-1 | Invariant holds — normal | ✅ |
| IC-2 | Invariant holds — boundary | ✅ |

## Cases

### IC-1: All Layer 2 crates satisfy the register_commands contract

- **Given:** Current 5-crate set (claude_assets, claude_version, claude_profile, claude_runner, claude_storage) all included via `build_registry()` in `src/lib.rs`
- **When:** `ast` binary is compiled with `--features enabled` and a representative command from each crate is invoked
- **Then:** Compilation succeeds (register_commands() exists in every crate); all 5 crates' commands are reachable; no unknown-command exit code 1

### IC-2: No orphan commands across Layer 2 crates

- **Given:** Every Layer 2 crate under the `enabled` feature has a `register_commands()` call in `build_registry()` and a `COMMANDS_YAML` path tracked by `build.rs`
- **When:** The set of commands registered by all `register_commands()` calls plus `register_static_commands()` is compared against commands defined in all Layer 2 YAML files and programmatic registrations
- **Then:** Every defined command appears in `ast`'s registry; no command is defined in a Layer 2 crate but unreachable from `ast`
