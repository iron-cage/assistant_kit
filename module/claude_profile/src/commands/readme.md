# src/commands/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations and command routine re-exports. |
| `cmd_args.rs` | Argument parsing and I/O error helpers for command handlers. |
| `cmd_context.rs` | Environment and credentials context resolution for command handlers. |
| `credentials.rs` | `.credentials.status` command routine. |
| `accounts.rs` | `.accounts` list command routine. |
| `accounts_render.rs` | Account list renderers and column-visibility for `.accounts`. |
| `account_ops.rs` | `.account.save/.use/.delete/.rotate` command routines. |
| `account_relogin.rs` | `.account.relogin` interactive re-authentication routine. |
| `account_renewal.rs` | `.account.renewal` subscription renewal check routine. |
| `account_inspect.rs` | `.account.inspect` per-account detail view routine. |
| `account_inspect_render.rs` | Formatting helpers for `.account.inspect` detail view. |
| `limits.rs` | `.account.limits` API rate-limit fetch routine. |
| `model.rs` | `.model` get/set session model routine. |
| `model_select.rs` | `.model.select` get/set/reset subprocess model preference routine. |
| `models.rs` | `.models` list available Claude models routine. |
| `token_paths.rs` | `.token.status` and `.paths` command routines. |
| `dot.rs` | `.` dot-shorthand command routine. |
