# claude_runner

CLI for executing Claude Code via builder pattern; YAML schema constants for command registration.

### Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|---------------|--------------|-------|--------------|
| claude_runner (lib) | YAML schema constants; `COMMANDS_YAML` path for consumers | — | YAML path constant, `VerbosityLevel` type | ❌ Process execution → `claude_runner_core` |
| clr (bin) | Standalone Claude Code CLI with session continuity by default | CLI args → process exit code | Arg parsing, session continuation, dry-run, help | ❌ Process execution → `claude_runner_core`<br>❌ Session paths → `claude_profile` |

### Scope

**Library (`src/lib.rs`):**
- `COMMANDS_YAML` constant — absolute path to `claude.commands.yaml`
- `VerbosityLevel` — newtype `u8` (0–5) for runner output gating
- Zero extra dependencies — always available regardless of features

**Binary (`src/main.rs`, requires `enabled` feature):**
- Mirrors Claude Code's native `--flag value` CLI syntax
- Session continuation (`-c`) applied by default; `--new-session` starts fresh
- Modes: interactive (default), print (`-p`), dry-run (`--dry-run`)
- Flags: `-p`, `--interactive`, `--new-session`, `--model`, `--verbose`, `--no-skip-permissions`,
  `--max-tokens`, `--session-dir`, `--dir`, `--dry-run`, `--trace`,
  `--system-prompt`, `--append-system-prompt`, `--verbosity`, `-h`
- Exit code propagation

### Files

| File / Directory | Responsibility |
|------------------|----------------|
| `Cargo.toml` | Crate manifest: lib + binary, optional feature-gated deps |
| `claude.commands.yaml` | Command definitions for `.claude` and `.claude.help` |
| `src/` | Library and binary source: `COMMANDS_YAML`, `VerbosityLevel`, `clr` CLI |
| `tests/` | CLI flag parsing, dry-run, verbosity, execution mode tests |
| `docs/` | CLI reference and design documentation |
| `changelog.md` | Notable changes by version |
| `verb/` | Shell scripts for each `do` protocol verb. |

### Architecture

```
claude_runner lib (YAML schema + COMMANDS_YAML constant + VerbosityLevel)
    └─ COMMANDS_YAML → path to claude.commands.yaml

clr binary (standalone CLI, mirrors claude's --flag syntax)
    └─ parse_args() → build_claude_command() → execute_interactive() / execute()
    └─ -c applied by default; --new-session to start fresh

YAML consumers (e.g. ast aggregator, build.rs)
    └─ aggregate claude.commands.yaml → PHF map (.claude, .claude.help)
```

### Separation of Concerns

- `claude_runner` (THIS crate): YAML schema + standalone CLI with default session continuity. Zero extra deps.
- `claude_runner_core`: `ClaudeCommand` builder — used by CLI binary.
- `claude_profile`: Session storage paths.

### Usage

**COMMANDS_YAML constant (lib, no features required):**
```rust
use claude_runner::COMMANDS_YAML;
aggregator.add(COMMANDS_YAML);
```

**CLI subprocess (binary, requires `enabled` feature):**
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

# Help
clr --help
```
