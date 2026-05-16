# API: Public API

### Scope

- **Purpose**: Document the programmatic interface of the claude_runner library surface.
- **Responsibility**: Specify COMMANDS_YAML, VerbosityLevel, and register_commands contracts, return types, and usage patterns.
- **In Scope**: COMMANDS_YAML constant value and usage, VerbosityLevel newtype semantics, register_commands no-op behavior.
- **Out of Scope**: CLI binary behavior (→ `feature/001_runner_tool.md`), dependency structure (→ `invariant/002_dep_constraints.md`).

### Abstract

The `claude_runner` library exposes three items: a compile-time path constant, a verbosity newtype, and an API-consistency no-op function. All three are designed for consumers that need to integrate Claude command definitions without depending on consumer workspace crates.

### Operations

#### `COMMANDS_YAML: &str`

Absolute path to `claude.commands.yaml`, computed at compile time via `env!("CARGO_MANIFEST_DIR")`. Stable across invocations on the same machine for the same build.

**Build-time aggregation (consumer `build.rs`):**
```
let claude_yaml = manifest_dir
  .join("../../claude_tools/dev/module/claude_runner/claude.commands.yaml");
base_commands.extend(load_yaml_and_transform(&claude_yaml));
```

**Runtime aggregation:**
```
aggregator.add(claude_runner::COMMANDS_YAML);
```

#### `VerbosityLevel`

Newtype wrapper over `u8` (valid range 0–5, default 3). Controls how much diagnostic output the `clr` binary emits.

| Level | Behavior |
|-------|----------|
| 0 | All runner diagnostic output suppressed |
| 1–3 | Progressively more output |
| 4 | Command preview printed to stderr before execution |
| 5 | Maximum verbosity |

`--dry-run` output is always shown regardless of `VerbosityLevel`. `--trace` mode prints env+command to stderr and then executes, independent of verbosity level.

#### `register_commands`

Gated behind the `enabled` feature. Empty-body function provided for API consistency with other Layer 2 crates that do runtime registration. Calling this function has no effect — actual registration of `.claude` and `.claude.help` commands is handled by build-time YAML aggregation via `COMMANDS_YAML`.

### Error Handling

The library surface has no fallible operations. `COMMANDS_YAML` is a `&'static str` constant. `register_commands` is a no-op and cannot fail.

### Compatibility Guarantees

- `COMMANDS_YAML` is stable as a `&'static str` constant.
- `VerbosityLevel` range (0–5) and default (3) are stable.
- `register_commands` will remain a no-op — its signature is stable but its empty body is by design.
- The `enabled` feature gate for `register_commands` is stable.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | CLI binary design that uses VerbosityLevel |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero consumer workspace dep rule that shapes this minimal library surface |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | COMMANDS_YAML and register_commands definitions |
| `../../src/verbosity.rs` | VerbosityLevel newtype implementation |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T49 flag parsing; covers --verbosity flag and VerbosityLevel parsing |
| `../../tests/commands_yaml_test.rs` | Validates COMMANDS_YAML path resolves to a readable, well-formed YAML file |
| `../../tests/verbosity_test.rs` | Full VerbosityLevel range, boundary, default, and method predicate coverage |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Public API section (COMMANDS_YAML, VerbosityLevel, register_commands), Consumers section |
