# assistant_kit

Layer 3 library facade re-exporting all Layer 2 full-featured coding agent crates.
Zero own logic — every public item originates from a Layer 2 crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest: five feature-gated optional workspace deps |
| `src/` | Feature-gated `pub mod` re-export modules for all five domains |
| `tests/` | Integration smoke tests verifying re-export paths per feature |
| `verb/` | Shell scripts for each `do` protocol verb. |

## Feature Flags

| Feature   | Activates                | Description                                   |
|-----------|--------------------------|-----------------------------------------------|
| `profile` | `claude_profile/enabled` | Account management, token status, CLI surface |
| `runner`  | `claude_runner/enabled`  | `ClaudeCommand` builder + CLI surface         |
| `version` | `claude_version/enabled` | Version detection, settings I/O, CLI surface  |
| `assets`  | `claude_assets/enabled`  | Symlink-based artifact installer CLI surface  |
| `storage` | `claude_storage/cli`     | Storage exploration CLI surface               |
| `full`    | all five above           | Everything                                    |
| `enabled` | `full`                   | Alias for `full`; conventional activation name |

## vs `dream`

| Crate | Layer | Re-exports | Use when |
|-------|-------|------------|----------|
| `dream` | 2 | `*_core` crates only — types and logic, no CLI | Embed core types without CLI overhead |
| `assistant_kit` | 3 | Full Layer 2 crates — types, logic, and CLI command surface | Embed complete CLI command surface as a library |

## Usage

```toml
[dependencies]
assistant_kit = { version = "^0.1", features = ["profile", "runner"] }
```

```rust,no_run
use assistant_kit::profile::ClaudePaths;
use assistant_kit::runner::strip_fences;
```

## Architecture

Layer 3 facade — depends on Layer 2 (`claude_profile`, `claude_runner`, `claude_version`,
`claude_assets`, `claude_storage`). No dependency on any binary or CLI framework crate
(`unilang`, `error_tools`). No own types, traits, functions, or error definitions.
