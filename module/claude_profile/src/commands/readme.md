# src/commands/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations and command routine re-exports. |
| `cmd_args.rs` | Argument parsing and I/O error helpers for command handlers. |
| `cmd_context.rs` | Environment and credentials context resolution for command handlers. |
| `credentials.rs` | `.credentials.status` command routine. |
| `accounts.rs` | `.accounts` list command routine. |
| `accounts_render.rs` | Account list renderers and column-visibility for `.accounts`. |
| `accounts_help.rs` | Grouped, `::`-aligned help rendering for `.accounts.help`. |
| `account_ops.rs` | `.account.save/.use/.delete/.rotate` command routines. |
| `account_relogin.rs` | `.account.relogin` interactive re-authentication routine. |
| `account_tag.rs` | `.account.tag` tag-set mutation routine. |
| `identity.rs` | `.tags`, `.identities`, `.identity.filter` Identity listing and filter routines. |
| `account_renewal.rs` | `.account.renewal` subscription renewal check routine. |
| `account_inspect.rs` | `.account.inspect` per-account detail view routine. |
| `account_inspect_render.rs` | Formatting helpers for `.account.inspect` detail view. |
| `limits.rs` | `.account.limits` API rate-limit fetch routine. |
| `model.rs` | `.model` unified session/subprocess model+effort get/set/reset routine. |
| `model_select.rs` | `.model.select` retirement stub — migration error to `.model scope::subprocess`. |
| `models.rs` | `.models` list available Claude models routine. |
| `token_paths.rs` | `.paths` command routine. |
| `dot.rs` | `.` dot-shorthand command routine. |
