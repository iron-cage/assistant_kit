# claude_runner

A batteries-included CLI runner for Claude Code, with YAML schema constants for command registration.

`clr` wraps the `claude` binary so automation callers don't have to script it themselves: session
continuity, a classified 3-tier retry hierarchy, a host-wide concurrency gate, structured output
rendering, and five-level layered configuration all ship built in. The crate's library surface
additionally exposes `COMMANDS_YAML` for consumers that register `clr` as a command.

### Responsibility Table

| Component | Responsibility | Input→Output | Scope | Out of Scope |
|--------|---------------|--------------|-------|--------------|
| claude_runner (lib) | YAML schema constants; `COMMANDS_YAML` path for consumers | — | YAML path constant | ❌ Process execution → `claude_runner_core` |
| clr / c / claude_runner (bin) | Standalone Claude Code CLI with session continuity by default | CLI args → process exit code | Arg parsing, session continuation, retries, gating, dry-run, help | ❌ Process execution → `claude_runner_core`<br>❌ Session paths → `claude_profile` |

### Scope

**Library (`src/lib.rs`):**
- `COMMANDS_YAML` constant — absolute path to `claude.commands.yaml`
- Zero extra dependencies — always available regardless of features

**Binary (`src/main.rs` / `src/bin/clr.rs` / `src/bin/c.rs`, requires `enabled` feature):**
- Mirrors Claude Code's native `--flag value` CLI syntax
- Session continuation (`-c`) applied by default; `--new-session` starts fresh
- Modes: interactive (default), print (`-p`), dry-run (`--dry-run`)
- Flags: `-p`, `--interactive`, `--new-session`, `--model`, `--verbose`, `--quiet`, `--no-skip-permissions`,
  `--max-tokens`, `--max-sessions`, `--session-dir`, `--dir`, `--dry-run`, `--trace`,
  `--system-prompt`, `--append-system-prompt`, `-h`
- Exit code propagation

### Features

- **Batteries included, nothing to script** — session continuity, retries, concurrency control, and
  configuration layering ship in the binary; automation callers just invoke `clr`.
- **Session continuity by default** — `-c` is injected automatically so consecutive invocations
  continue the same conversation; `--new-session` opts out.
- **Three execution modes** — interactive REPL (default), non-interactive print mode (`-p`, for
  capturing output), and `--dry-run` (prints the exact command without executing).
- **3-tier retry hierarchy** — 8 classified error types (Transient, Account, Auth, Service, Process,
  Validation, Runner, Unknown), each resolved through override → per-class → default tiers, with
  per-class `CLR_*` env var fallbacks. See [docs/feature/003_retry_hierarchy.md](docs/feature/003_retry_hierarchy.md).
- **Session concurrency gate** — `--max-sessions` caps concurrent `claude` processes host-wide via an
  atomic slot-reservation scheme (race-free under concurrent launches); `0` disables the gate. See
  [docs/invariant/012_gate_slot_atomicity.md](docs/invariant/012_gate_slot_atomicity.md).
- **Layered configuration** — five-level precedence (CLI flag → `--args-file`/`CLR_ARGS_FILE` JSON →
  `CLR_*` env var → `.clr.toml`/`~/.clr/config.toml` → built-in default). See
  [Configuration](#configuration) below.
- **Automatic journaling** — every execution boundary (subprocess exit, credential ops, gate waits,
  retries, timeouts, interactive start/end) is recorded via `claude_journal`, best-effort and
  non-fatal. See [docs/feature/002_journaling_integration.md](docs/feature/002_journaling_integration.md).
- **Structured output rendering** — summary mode (default) renders a human-readable line from the
  JSON result envelope; `--output-style raw` bypasses rendering; `--expect` validates output against
  expected values with configurable retry/fallback strategies.
- **Fleet visibility** — `clr ps` lists active sessions and queued gate waiters; `clr kill <pid>`
  terminates a session; `clr scope` prints the 6 `CLAUDE_*` path variables for a directory.
- **Isolated one-shot execution** — `clr isolated` runs a task in a sandboxed temp-HOME, separate
  from the caller's normal session and credentials; `clr refresh` refreshes Claude Code OAuth
  credentials the same way.

### Installation

Both binary targets require the `enabled` feature (on by default):

```sh
cargo install --path . --features enabled   # or: cargo build --release
```

This produces three identical binaries — `clr` (primary), `c` (ultra-short alias), and
`claude_runner` (full name) — all delegating to the same `claude_runner::run_cli()` entry point.
The library target (`COMMANDS_YAML`) has zero extra dependencies and builds without any feature flags.

### Usage

**CLI subcommands:**

| Subcommand | Purpose |
|------------|---------|
| `clr` / `clr run` | Default execution — interactive or print mode (runs when no subcommand matches) |
| `clr ask` | Semantic alias for `run` (same behavior, `ask`-specific help text) |
| `clr isolated` | One-shot task in an isolated temp-HOME sandbox |
| `clr refresh` | Refresh Claude Code OAuth credentials (isolated one-shot) |
| `clr ps` | List active Claude Code sessions and queued gate waiters |
| `clr kill <pid>` | Terminate a running Claude Code session |
| `clr tools` | List the 26 Claude Code built-in tools |
| `clr scope` | Print the 6 `CLAUDE_*` path variables for a directory |
| `clr help` | Show help text |

```sh
# Interactive mode (default — continues previous session)
clr                                        # REPL
clr "Fix the bug"                          # interactive with prompt
clr "Fix bug" --dir /path                  # continue in specific dir

# Non-interactive print mode (continues previous session)
clr -p "Explain this" --model sonnet       # capture output
clr -p "Run tests" --max-tokens 50000      # with token limit

# New session (explicit fresh start)
clr --new-session "Start fresh analysis"
clr --new-session -p "Review from scratch" --model opus

# Dry-run (preview command without executing; shows -c in output)
clr --dry-run "test"                       # show what would run
clr --dry-run --verbose "fix it"           # preview with all flags

# Concurrency gate and fleet visibility
clr -p "task" --max-sessions 3             # cap concurrent claude processes at 3
clr ps                                     # active sessions + queued waiters
clr kill 12345                             # terminate a session by PID

# Help
clr --help
```

**COMMANDS_YAML constant (lib, no features required):**
```rust
use claude_runner::COMMANDS_YAML;
aggregator.add( COMMANDS_YAML );
```

### Configuration

Parameters resolve through five levels, highest precedence first:

1. **CLI flag** — wins unconditionally when provided
2. **JSON config** — `--args-file <PATH>`, `CLR_ARGS_FILE`, or a piped JSON object on stdin
3. **`CLR_*` env var** — per-parameter environment variable fallback
4. **Config file** — project `.clr.toml` (cwd), then user `~/.clr/config.toml` (or
   `$CLR_CONFIG_DIR/config.toml`); project wins when both set the same key
5. **Built-in default**

```sh
echo 'model = "claude-opus-4-8"' > .clr.toml
clr --dry-run "task"                 # shows: claude --model claude-opus-4-8 ...
clr --model sonnet --dry-run "task"  # CLI wins; config value ignored
```

Use `--dry-run` or `--trace` to see effective values after every tier has applied. Full reference:
[docs/cli/env_param.md](docs/cli/env_param.md), [docs/cli/config_param.md](docs/cli/config_param.md),
[docs/feature/004_json_config.md](docs/feature/004_json_config.md).

### Files

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest: lib + binary, optional feature-gated deps |
| `claude.commands.yaml` | Command definitions for `.claude` and `.claude.help` |
| `src/` | Library and binary source: `COMMANDS_YAML`, `clr` CLI |
| `tests/` | CLI flag parsing, dry-run, quiet gate, execution mode tests |
| `docs/` | CLI reference and design documentation |
| `changelog.md` | Notable changes by version |
| `rulebook.md` | Local code-style exception: mechanical dispatch function length |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `runbox/` | Container environment scripts for test execution |
| `bug/` | Bug reports: open and closed defects for this crate |
| `task/` | Task tracking: verified and completed work items. |

### Architecture

```
claude_runner lib (YAML schema + COMMANDS_YAML constant)
    └─ COMMANDS_YAML → path to claude.commands.yaml

clr binary (standalone CLI, mirrors claude's --flag syntax)
    └─ parse_args() → build_claude_command() → execute_interactive() / execute()
    └─ -c applied by default; --new-session to start fresh
    └─ wait_for_session_slot() gate → retry-classified execution → journal + render

YAML consumers (e.g. ast aggregator, build.rs)
    └─ aggregate claude.commands.yaml → PHF map (.claude, .claude.help)
```

### Separation of Concerns

- `claude_runner` (THIS crate): YAML schema + standalone CLI with default session continuity,
  retries, gating, and configuration. Zero extra deps for the library target.
- `claude_runner_core`: `ClaudeCommand` builder — used by CLI binary.
- `claude_profile`: Session storage paths.

### Documentation

| Location | Covers |
|----------|--------|
| [docs/feature/](docs/feature/) | Runner tool design, journaling, retry hierarchy, JSON config, session path resolution, CLI design |
| [docs/invariant/](docs/invariant/) | Enforced invariants (default flags, dependency constraints, gate slot atomicity, etc.) |
| [docs/api/](docs/api/) | Public API contract (`COMMANDS_YAML`, `register_commands`) |
| [docs/cli/](docs/cli/) | Full parameter reference: env vars, config file, command defaults |
