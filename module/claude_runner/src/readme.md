# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Library API: COMMANDS_YAML constant, register_commands shim, and run_cli() entry point. |
| `cli/` | CLI module: argument parsing, env fallbacks, command building, subcommand dispatch, and execution. |
| `main.rs` | `claude_runner` binary entry point; delegates to `run_cli()`. |
| `bin/` | Binary aliases: `clr` and `c` entry points (thin wrappers). |
