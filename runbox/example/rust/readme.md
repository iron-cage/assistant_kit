# rust

Minimal Rust binary proving universal runbox usage for the native Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Library source: `pub fn add(a, b)`. |
| `src/main.rs` | Binary entry point (prints result of `add(2, 3)`). |
| `tests/` | Integration test suite (3 tests via `cargo test`). |
| `run/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `Cargo.toml` | Package manifest: lib + bin targets. |

## Quick Start

```bash
./run/runbox .build          # build container image
./run/runbox .test           # run tests
./run/runbox .lint           # run cargo clippy
./run/runbox .run            # run binary (prints: add(2, 3) = 5)
./run/runbox .test.offline   # offline tests (uses seeded target volume)
./run/runbox .list           # list tests
./run/runbox .shell          # interactive shell
```
