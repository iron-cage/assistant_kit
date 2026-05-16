# cli_fmt

Local copy of `cli_fmt` from the `wtools` workspace, providing the `cli_help_template` feature for Docker build context isolation.

| File | Responsibility |
|------|---------------|
| Cargo.toml | Crate manifest with cli_help_template feature only |
| src/lib.rs | Crate root re-exporting help module |
| src/help.rs | CliHelpTemplate renderer, CliHelpStyle, CliHelpData types |
