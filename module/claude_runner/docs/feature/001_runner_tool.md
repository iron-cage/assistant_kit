# Feature: Runner Tool

### Scope

- **Purpose**: Document the clr CLI tool design including execution modes, default flag injection, and the YAML library surface.
- **Responsibility**: Describe the two roles of claude_runner (CLI binary and YAML library), invocation modes, and flag behavior.
- **In Scope**: clr execution modes, automatic `-c`, `--dangerously-skip-permissions`, `--chrome` injection, `"\n\nultrathink"` message suffix default, YAML library role, mode selection logic.
- **Out of Scope**: Dependency constraints (→ `invariant/002_dep_constraints.md`), public API contracts (→ `api/001_public_api.md`).

### Design

claude_runner serves two distinct consumers from one crate:

**YAML library consumer:** The library surface exposes `COMMANDS_YAML` — an absolute path (computed at compile time via `env!("CARGO_MANIFEST_DIR")`) to `claude.commands.yaml`. Consumers such as `dream` aggregate this YAML at compile time via `build.rs` to build a PHF static command registry for `.claude` and `.claude.help` commands. The library has zero consumer workspace dependencies.

**CLI binary (`clr`):** The `clr` binary translates `--flag value` syntax to `ClaudeCommand` builder calls and executes Claude Code via `claude_runner_core`. It acts as the user-facing runner for both interactive and non-interactive use.

**Execution modes:** See [001_command.md](../cli/001_command.md) for the full invocation mode table.

**Default flag injection:** See [invariant/001_default_flags.md](../invariant/001_default_flags.md) for the complete default injection rules and opt-out mechanisms.

**Verbosity gate:** The `--verbosity <0-5>` flag (default 3) controls how much runner diagnostic output is emitted. At level 0 all diagnostic output is suppressed. At level 4 a command preview is printed to stderr before execution. `--dry-run` output is always shown regardless of verbosity level.

**Trace mode:** `--trace` prints environment variables and the full command to stderr (like `set -x`), then executes normally. This is independent of verbosity level.

**Stdin file piping:** `--file <PATH>` opens the given file and pipes its content as standard input to the `claude` subprocess. Equivalent to `cat file | clr -p "..."` but without a shell pipeline. The file is opened at spawn time; a non-readable path causes `clr` to exit with an error.

**Output fence stripping:** `--strip-fences` post-processes captured stdout after the subprocess exits: the first and last markdown code fence lines (`` ``` `` with optional language tag) are removed and the content between them is emitted unchanged. If no fence pair is found, stdout passes through unmodified.

**CLAUDECODE removal:** `clr` removes the `CLAUDECODE` environment variable from the subprocess environment before spawning (default-on). This prevents the subprocess from detecting a parent Claude Code session, which would alter its behaviour. Use `--keep-claudecode` to opt out and preserve `CLAUDECODE` in the subprocess environment.

**Separation of concerns:** `clr` owns CLI flag translation and automation defaults only. Process execution is delegated to `claude_runner_core`. Session storage paths come from `claude_profile` (via `--session-dir` flag passthrough or resolved externally).

### APIs

| File | Relationship |
|------|--------------|
| [api/001_public_api.md](../api/001_public_api.md) | COMMANDS_YAML and VerbosityLevel public API |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_default_flags.md](../invariant/001_default_flags.md) | Default flag injection rules and opt-out mechanism |
| [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | run_cli() entry point; mode dispatch (run_print_mode, run_interactive) |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T49 flag parsing; --interactive, --print mode dispatch coverage |
| `../../tests/dry_run_test.rs` | Validates dry-run preview output including all injected flags |
| `../../tests/execution_mode_test.rs` | E01–E13 live mode dispatch via fake claude binary |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Purpose, Architecture, Modes, Default Flags Principle, CLI Flags, Separation of Concerns |
