# dream

Layer 2 library facade re-exporting all Layer 0–1 core crates under feature-gated modules.
Zero own logic — every public item originates from a core crate.

## Responsibility Table

| File/Dir | Responsibility |
|----------|---------------|
| `Cargo.toml` | Crate manifest: seven feature-gated optional workspace deps |
| `src/lib.rs` | Feature-gated `pub mod` re-exports + crate-level Feature Flags table |
| `docs/` | Crate doc entities: aggregation feature spec and zero-own-logic invariant |
| `tests/` | Integration smoke tests verifying re-export paths per feature |

## Feature Flags

| Feature | Activates | Description |
|---------|-----------|-------------|
| `common` | `claude_core` | Path topology (`ClaudePaths`) and process utilities |
| `storage` | `claude_storage_core` | Zero-dep JSONL parser for `~/.claude/` storage |
| `profile` | `claude_profile_core` | Token status detection and account credential management |
| `runner` | `claude_runner_core` | `ClaudeCommand` builder for programmatic Claude Code execution |
| `version` | `claude_version_core` | Version detection, settings I/O, and install helpers |
| `assets` | `claude_assets_core` | Symlink-based artifact installer |
| `full` | all six above | All domain modules in a single dependency |
| `enabled` | `full` | Alias for `full`; conventional workspace activation name |

## Usage

```toml
# Cargo.toml
[dependencies]
dream = { version = "~1.0", features = ["profile", "runner"] }
```

```rust,no_run
use dream::profile::token::TokenStatus;
use dream::runner::ClaudeCommand;
```

## Architecture

Layer 2 facade — depends on Layer 0 (`claude_core`) and Layer 1
(`claude_profile_core`, `claude_runner_core`, `claude_version_core`, `claude_assets_core`) plus the
out-of-hierarchy `claude_storage_core`. No dependency on any Layer 2 CLI crate or
Layer 3 super-app. No own types, traits, functions, or error definitions.
