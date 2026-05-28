# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Library API: COMMANDS_YAML constant, VerbosityLevel re-export, register_commands shim, and run_cli() entry point. |
| `cli/` | CLI module: argument parsing, env fallbacks, subcommand dispatch, and command execution. |
| `cli/parse.rs` | Structs, parse functions, env helpers, and apply_env_vars for all subcommands. |
| `cli/credential.rs` | Isolated/refresh execution: emit_credential_trace, run_isolated_command, run_refresh_command. |
| `cli/mod.rs` | Coordinator: help printing, command building, run/ask dispatch, strip_fences, unit tests. |
| `main.rs` | `claude_runner` binary entry point; delegates to `run_cli()`. |
| `verbosity.rs` | `VerbosityLevel` newtype: output gating with semantic level methods. |
| `bin/` | Binary aliases: `clr` and `c` entry points (thin wrappers). |
