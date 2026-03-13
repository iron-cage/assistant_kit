# Feature: Runner Tool

### Scope

- **Purpose**: Document the clr CLI tool design including execution modes, default flag injection, and the YAML library surface.
- **Responsibility**: Describe the two roles of claude_runner (CLI binary and YAML library), invocation modes, and flag behavior.
- **In Scope**: clr execution modes, automatic `-c` and `--dangerously-skip-permissions` injection, YAML library role, mode selection logic.
- **Out of Scope**: Dependency constraints (→ `invariant/002_dep_constraints.md`), public API contracts (→ `api/001_public_api.md`).

### Design

claude_runner serves two distinct consumers from one crate:

**YAML library consumer:** The library surface exposes `COMMANDS_YAML` — an absolute path (computed at compile time via `env!("CARGO_MANIFEST_DIR")`) to `claude.commands.yaml`. Consumers such as `dream` aggregate this YAML at compile time via `build.rs` to build a PHF static command registry for `.claude` and `.claude.help` commands. The library has zero willbe dependencies.

**CLI binary (`clr`):** The `clr` binary translates `--flag value` syntax to `ClaudeCommand` builder calls and executes Claude Code via `claude_runner_core`. It acts as the user-facing runner for both interactive and non-interactive use.

**Execution modes:**

| Invocation | Mode | Execution Path |
|------------|------|----------------|
| `clr` | Interactive REPL | `execute_interactive()` + `-c` |
| `clr "Fix bug"` | Print (default) | `execute()` + `--print` + `-c` |
| `clr -p "Fix bug"` | Print (explicit) | `execute()` + `--print` + `-c` |
| `clr --interactive "Fix bug"` | Interactive | `execute_interactive()` + `-c` |
| `clr --dry-run "Fix bug"` | Preview only | `describe()` / `describe_env()` output |
| `clr --trace "Fix bug"` | Trace | `describe_env()` + `describe()` to stderr, then `execute()` |
| `clr --new-session "Fix bug"` | Fresh session, print | `execute()` + `--print` (no `-c`) |

**Default flag injection:** `-c` (continue conversation) is applied automatically to every invocation. Use `--new-session` to start a fresh session instead. `--dangerously-skip-permissions` is injected by default to prevent automation stalling on permission prompts. Use `--no-skip-permissions` to disable this per invocation.

**Verbosity gate:** The `--verbosity <0-5>` flag (default 3) controls how much runner diagnostic output is emitted. At level 0 all diagnostic output is suppressed. At level 4 a command preview is printed to stderr before execution. `--dry-run` output is always shown regardless of verbosity level.

**Trace mode:** `--trace` prints environment variables and the full command to stderr (like `set -x`), then executes normally. This is independent of verbosity level.

**Separation of concerns:** `clr` owns CLI flag translation and automation defaults only. Process execution is delegated to `claude_runner_core`. Session storage paths come from `claude_profile` (via `--session-dir` flag passthrough or resolved externally).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| api | [api/001_public_api.md](../api/001_public_api.md) | COMMANDS_YAML and VerbosityLevel public API |
| invariant | [invariant/001_default_flags.md](../invariant/001_default_flags.md) | Default flag injection rules and opt-out mechanism |
| invariant | [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero willbe deps, binary deps gated by enabled |
| source | `../../src/main.rs` | clr binary entry point and mode dispatch |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Purpose, Architecture, Modes, Default Flags Principle, CLI Flags, Separation of Concerns |
