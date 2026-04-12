# claude_assets/src

| File | Responsibility |
|------|----------------|
| `lib.rs` | `COMMANDS_YAML`, `register_commands()`, `run_cli()` entry points |
| `commands.rs` | Command routines: list, install, uninstall, kinds |
| `adapter.rs` | argv-to-unilang token conversion; alias and bool normalisation |
| `main.rs` | `claude_assets` binary entry point |
| `bin/` | Binary alias targets |
