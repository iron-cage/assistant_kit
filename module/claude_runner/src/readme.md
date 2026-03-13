# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Public API: `COMMANDS_YAML` constant + `VerbosityLevel` re-export. |
| `main.rs` | CLI binary: arg parsing, `ClaudeCommand` builder, execute. |
| `verbosity.rs` | `VerbosityLevel` newtype: output gating with semantic level methods. |
