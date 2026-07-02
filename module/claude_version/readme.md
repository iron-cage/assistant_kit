# claude_version

Claude Code version manager: install, upgrade, and session lifecycle.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `docs/` | Design and CLI documentation (feature/, algorithm/, pattern/, cli/) |
| `src/` | Binary and library source code |
| `tests/` | Unit and integration test suite |
| `Cargo.toml` | Crate manifest |
| `unilang.commands.yaml` | YAML command metadata for all manager commands (not aggregated by build.rs) |
| `changelog.md` | Notable changes by version |
| `runbox/` | Container runner integration scripts and config |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `license` | License text for this crate |

## Scope

- **In scope:** Installing and upgrading Claude Code; version aliasing and drift guard; process inspection and termination; settings and config management.
- **Out of scope:** Running Claude sessions; prompt handling; account authentication (delegated to `claude_core`).

## Usage

```bash
# Check current installation state
clv .status

# Install the latest Claude Code version
clv .version.install

# Read a settings key
clv .settings.get key::model

# Inspect available params and current values
clv .params
```
