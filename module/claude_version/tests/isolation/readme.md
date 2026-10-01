# tests/isolation/

Spawn guard for both test binaries. `tests/cli.rs` and `tests/cli_args_test.rs` each
include `spawn_guard.rs` via `#[ path ]`, and their subprocess helpers call it before
every `claude_version` spawn. This directory holds no tests of its own.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `spawn_guard.rs` | Check the container and `HOME` before each binary spawn |
