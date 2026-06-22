# rust

Minimal Rust binary proving universal container runner usage for the native Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Library source: `pub fn add(a, b)`. |
| `src/main.rs` | Binary entry point (prints result of `add(2, 3)`). |
| `tests/` | Integration test suite (3 tests via `cargo test`). |
| `run/` | Container integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `Cargo.toml` | Package manifest: lib + bin targets. |

## Quick Start

```bash
./verb/.build          # build container image
./verb/.test           # run tests
./verb/.lint           # run cargo clippy
./verb/.run            # run binary (prints: add(2, 3) = 5)
./verb/.test.offline   # offline tests (uses seeded target volume)
./verb/.list           # list tests
./verb/.shell          # interactive shell
```
