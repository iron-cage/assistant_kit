# Invariant: Default Flags

### Scope

- **Purpose**: Document the automatic flag injection behavior that must be maintained across all clr invocations.
- **Responsibility**: State which flags are injected by default, their opt-out mechanism, and why the defaults exist.
- **In Scope**: Automatic `-c` injection, `--dangerously-skip-permissions` default-on, `--chrome` builder default, `--new-session` override, `--no-skip-permissions` opt-out.
- **Out of Scope**: Dependency constraints (→ `invariant/002_dep_constraints.md`), execution mode behavior (→ `feature/001_runner_tool.md`).

### Invariant Statement

`clr` must inject the following flags on every invocation unless explicitly overridden:

| Flag | Default | Override | Rationale |
|------|---------|----------|-----------|
| `-c` (continue conversation) | ON | `--new-session` | Automation expects session continuity by default |
| `--dangerously-skip-permissions` | ON | `--no-skip-permissions` | Automation pipelines must not stall on permission prompts |
| `--chrome` | ON | none at clr level | Browser context is essential for web-aware automation |

These defaults are intentional and must not be removed without explicit design decision. They represent the automation-optimized defaults for the `clr` runner.

### Enforcement Mechanism

The flag injection is implemented at two layers:
- `-c` and `--dangerously-skip-permissions`: injected explicitly by `build_claude_command()` (the function that translates `CliArgs` to a `ClaudeCommand` builder). Added unconditionally unless the corresponding opt-out flag is present in `CliArgs`.
- `--chrome`: injected via `ClaudeCommand::new()` builder default (`chrome: Some(true)`). No clr-level opt-out exists — callers using the Rust API can override with `with_chrome(None)` or `with_chrome(Some(false))`.

`--dangerously-skip-permissions` is no longer user-facing as a positive flag. Users disable it via `--no-skip-permissions`. This prevents confusion between "skip permissions" as an intentional choice vs. the default behavior.

Dry-run mode (`--dry-run`) shows the injected flags in its output — the preview reflects the actual command that would be run, including all default injections.

### Violation Consequences

If either default injection is removed:
- Interactive automation scripts stall waiting for permission prompts (skip-permissions removed)
- Each invocation starts a new session, losing conversation context (continuation removed)
- Automation pipelines that depend on these defaults will behave differently without a version change

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Execution modes that consume these injected defaults |
| source | `../../src/main.rs` | build_claude_command() flag injection implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Default Flags Principle, Modes table, CLI Flags (--new-session, --no-skip-permissions) |
