# src/commands/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations and command routine re-exports. |
| `shared.rs` | Output helpers shared across command routines. |
| `credentials.rs` | `.credentials.status` command routine. |
| `accounts.rs` | `.accounts` list command routine. |
| `account_ops.rs` | `.account.save/.use/.delete/.rotate` command routines. |
| `account_relogin.rs` | `.account.relogin` interactive re-authentication routine. |
| `account_renewal.rs` | `.account.renewal` subscription renewal check routine. |
| `account_inspect.rs` | `.account.inspect` per-account detail view routine. |
| `limits.rs` | `.account.limits` API rate-limit fetch routine. |
| `model.rs` | `.model` get/set session model routine. |
| `token_paths.rs` | `.token.status` and `.paths` command routines. |
| `dot.rs` | `.` dot-shorthand command routine. |
