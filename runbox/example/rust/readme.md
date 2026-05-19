# rust

Minimal Rust binary proving universal runbox usage for the native Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Library source: `pub fn add(a, b)`. |
| `src/main.rs` | Binary entry point (prints result of `add(2, 3)`). |
| `tests/` | Integration test suite (3 tests via `cargo test`). |
| `runbox/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `Cargo.toml` | Package manifest: lib + bin targets. |

## Quick Start

```bash
./runbox/runbox .build          # build container image
./runbox/runbox .test           # run tests
./runbox/runbox .lint           # run cargo clippy
./runbox/runbox .run            # run binary (prints: add(2, 3) = 5)
./runbox/runbox .test.offline   # offline tests (uses seeded target volume)
./runbox/runbox .list           # list tests
./runbox/runbox .shell          # interactive shell
```
