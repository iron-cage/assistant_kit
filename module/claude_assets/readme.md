# claude_assets

| File | Responsibility |
|------|----------------|
| src/lib.rs | COMMANDS_YAML, register_commands(), run_cli() entry points |
| src/commands.rs | Command routines: list, install, uninstall, kinds |
| src/main.rs | claude_assets binary entry point |
| src/bin/cla.rs | cla alias binary entry point |
| unilang.commands.yaml | CLI command definitions metadata |
| tests/cli.rs | Integration tests via assert_cmd |
