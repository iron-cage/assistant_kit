# fake_claude_bin

Shared fake-`claude`-binary fixture for subprocess integration tests.

## Responsibility

| File | Responsibility |
|------|---------------|
| readme.md | This directory guide |
| mod.rs | `fake_claude_dir()` — temp dir with executable fake `claude` script plus augmented `PATH` |

## Usage

Each consuming test binary includes the module directly (Cargo does not compile
`tests/` subdirectories as test binaries):

```rust
mod fake_claude_bin;
use fake_claude_bin::fake_claude_dir;

let ( _dir, path_val ) = fake_claude_dir( "printf 'hello'" );
```

Keep the returned `TempDir` binding alive for the test's duration — dropping it
deletes the fake binary.

Consumers: `bug_243_test.rs`, `stdin_file_test.rs`, `control_stderr_drain_test.rs`,
`isolated_test.rs`.
