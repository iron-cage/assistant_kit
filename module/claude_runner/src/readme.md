# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Library API: COMMANDS_YAML constant, VerbosityLevel re-export, register_commands shim, and run_cli() entry point. |
| `cli.rs` | CLI argument parsing, subcommand dispatch, execution modes, and command builder logic. |
| `main.rs` | `claude_runner` binary entry point; delegates to `run_cli()`. |
| `verbosity.rs` | `VerbosityLevel` newtype: output gating with semantic level methods. |
| `bin/` | `clr` binary alias entry point (thin wrapper). |
