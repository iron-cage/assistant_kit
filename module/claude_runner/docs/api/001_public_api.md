# API: Public API

### Scope

- **Purpose**: Document the programmatic interface of the claude_runner library surface.
- **Responsibility**: Specify COMMANDS_YAML and register_commands contracts, return types, and usage patterns.
- **In Scope**: COMMANDS_YAML constant value and usage, register_commands no-op behavior.
- **Out of Scope**: CLI binary behavior (→ `feature/001_runner_tool.md`), dependency structure (→ `invariant/002_dep_constraints.md`).

### Abstract

The `claude_runner` library exposes two items: a compile-time path constant and an API-consistency no-op function. Both are designed for consumers that need to integrate Claude command definitions without depending on consumer workspace crates.

### Operations

#### `COMMANDS_YAML: &str`

Absolute path to `claude.commands.yaml`, computed at compile time from the crate manifest directory. Stable across invocations on the same machine for the same build.

**Build-time aggregation:** Pass the path to a YAML loader in a build script to incorporate `claude_runner` commands into a static command registry.

**Runtime aggregation:**
```
aggregator.add(claude_runner::COMMANDS_YAML);
```

#### `register_commands`

Gated behind the `enabled` feature. Empty-body function provided for API consistency with other Layer 2 crates that do runtime registration. Calling this function has no effect — actual registration of `.claude` and `.claude.help` commands is handled by build-time YAML aggregation via `COMMANDS_YAML`.

### Error Handling

The library surface has no fallible operations. `COMMANDS_YAML` is a `&'static str` constant. `register_commands` is a no-op and cannot fail.

### Compatibility Guarantees

- `COMMANDS_YAML` is stable as a `&'static str` constant.
- `register_commands` will remain a no-op — its signature is stable but its empty body is by design.
- The `enabled` feature gate for `register_commands` is stable.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | CLI binary design that uses COMMANDS_YAML and register_commands |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero consumer workspace dep rule that shapes this minimal library surface |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | COMMANDS_YAML and register_commands definitions |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T35 flag parsing coverage |
| `../../tests/cli_args_ext_test.rs` | T36–T49, S58–S79 extended flag parsing coverage |
| `../../tests/commands_yaml_test.rs` | Validates COMMANDS_YAML path resolves to a readable, well-formed YAML file |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Public API section (COMMANDS_YAML, register_commands), Consumers section |
