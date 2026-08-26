# src/

This directory contains the core implementation of the `claude_runner_core` crate, providing builder pattern API for executing Claude Code commands programmatically.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate entry point and public API surface |
| `command/` | ClaudeCommand builder split into per-tier parameter modules |
| `control.rs` | Bidirectional control-protocol session over stream-json stdio |
| `exit_code.rs` | Classify subprocess exit codes/stderr into `ErrorKind` |
| `isolated.rs` | One-shot run with isolated temp HOME and injected credentials |
| `types.rs` | Enum type definitions and conversions |
| `process.rs` | Scan `/proc` for Claude processes; send SIGTERM/SIGKILL |
| `ps_table.rs` | Render a `ProcessInfo` slice as a table (feature `ps_table`) |
| `session_dir.rs` | Directory-based session isolation for invocations |

## Organization (9 entries)

Files organized by responsibility following Rust module conventions.

### Module Structure

```
src/
├── lib.rs              # Crate root, public API
├── command/            # ClaudeCommand builder (split into 4 files)
│   ├── mod.rs          # Struct def, execution methods, describe helpers
│   ├── params_core.rs  # Tier 1 critical parameters
│   ├── params_security.rs  # Tier 2 security-sensitive parameters
│   └── params_extended.rs  # Tier 3+ optional parameters
├── control.rs          # Bidirectional control-protocol session (stream-json)
├── exit_code.rs        # Exit-code/stderr → ErrorKind classification
├── isolated.rs         # run_isolated(): temp-HOME one-shot execution
├── types.rs            # ActionMode, LogLevel enums
├── process.rs          # /proc scanner, signal sending
├── ps_table.rs         # ProcessInfo table rendering (feature `ps_table`)
└── session_dir.rs      # Session directory isolation
```

### Scope

**In Scope:**
- Builder pattern API for Claude Code command construction
- Environment variable automation (tier 1 defaults: bash_timeout=3.6M, bash_max_timeout=7.2M, auto_continue=true, telemetry=false, max_output_tokens=128K)
- Type safety via enums (ActionMode, LogLevel)
- Private field encapsulation (prevents direct construction)
- Command execution via `execute()`/`execute_interactive()`, plus lower-level `spawn_*()` entry points for direct process/stream control (see `command/mod.rs` doc comment for the full set)
- Test-only helpers for verification without actual execution
- `/proc` scanning for running Claude Code processes
- Signal delivery (SIGTERM, SIGKILL) to Claude processes
- Process-list table rendering for downstream `.ps`-style consumers (feature `ps_table`)

**Out of Scope:**
- Session lifecycle management (→ claude_profile crate)
- Context injection from consumer_runner (→ dream_agent crate)
- Interactive terminal UI (→ terminal-based tools)
- Configuration hierarchy (→ config_hierarchy crate)

### Invariants

Formally documented and test-enforced — see `docs/invariant/readme.md` for the full index:
- [`001_single_execution_point.md`](../docs/invariant/001_single_execution_point.md) — all `Command::new("claude")` calls centralize in `build_command()`; `execute()`, `execute_interactive()`, `spawn_piped()`, `spawn_tty()`, and `spawn_control_session()` all resolve through it. Enforced by `tests/responsibility_single_execution_point_test.rs`.

### Test Coverage

Comprehensive test suite in `tests/` directory:
- Builder pattern API (4 test files): edge cases, methods, defaults, environment variables
- Type definitions (1 test file): enum conversions and defaults
- Migration validation (2 test files): factory pattern removal, single execution point
- Verification framework (5 test files): 231 validation assertions across 6 layers
- Inspection methods (1 test file): describe() and describe_env()
- Execution output (1 test file): ExecutionOutput struct and Display
- Skip permissions (1 test file): --dangerously-skip-permissions flag
- Manual execution (2 test files): real Claude binary tests (skipped in CI)
- **Total**: 43 test files, all passing; see tests/readme.md for the complete Responsibility Table

See `tests/readme.md` for complete test documentation.
