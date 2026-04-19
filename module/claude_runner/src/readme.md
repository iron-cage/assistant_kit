# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | CLI parsing pipeline, execution routing, and library API constants. |
| `main.rs` | `claude_runner` binary entry point; delegates to `run_cli()`. |
| `verbosity.rs` | `VerbosityLevel` newtype: output gating with semantic level methods. |
| `bin/` | `clr` binary alias entry point (thin wrapper). |
