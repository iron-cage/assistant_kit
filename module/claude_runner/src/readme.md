# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Library API: COMMANDS_YAML constant, VerbosityLevel re-export, register_commands shim, and run_cli() entry point. |
| `cli/` | CLI module: argument parsing, env fallbacks, command building, subcommand dispatch, and execution. |
| `main.rs` | `claude_runner` binary entry point; delegates to `run_cli()`. |
| `verbosity.rs` | `VerbosityLevel` newtype: output gating with semantic level methods. |
| `bin/` | Binary aliases: `clr` and `c` entry points (thin wrappers). |
