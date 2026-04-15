# dream

Rust workspace for programmatic Claude Code integration: credential management, session storage, and process execution.

## Quick Start

```bash
cargo build --release
clt .help          # list all commands
clp .account.list  # list saved accounts
clr "write a test" # run Claude Code
```

## Structure

| Path | Responsibility |
|------|----------------|
| `module/` | Thirteen workspace crates (see Crates below) |
| `docs/` | Workspace doc entities: feature, invariant, pattern, integration, Claude Code knowledge |
| `task/` | Task tracking: active, completed, backlog |
| `vision.md` | Project vision, design rationale, and open problems |
| `locales.md` | Locale and internationalisation notes |
| `Cargo.toml` | Workspace manifest: members, lints, shared dependencies |

## Crates

| Crate | Cmd | Layer | Responsibility |
|---|---|---|---|
| `claude_common` | — | 0 | Shared primitives: `ClaudePaths`, process utilities |
| `claude_storage_core` | — | * | Zero-dep JSONL parser for `~/.claude/`; path encoding |
| `claude_profile_core` | — | 1 | Token status + account domain logic |
| `claude_version_core` | — | 1 | Version detection, install, settings domain helpers |
| `claude_runner_core` | — | 1 | `ClaudeCommand` builder + single process execution point |
| `claude_assets_core` | — | 1 | Symlink-based artifact installer domain logic |
| `claude_profile` | `clp` | 2 | Account management, token status, `~/.claude/` paths |
| `claude_storage` | `clg` | 2 | CLI for exploring Claude Code filesystem storage |
| `claude_runner` | `clr` | 2 | Claude Code execution with session continuity |
| `claude_version` | `clv` | 2 | Install, version, session, and settings management |
| `claude_assets` | `cla` | 2 | Install Claude Code artifacts (rules, skills, commands) via symlinks |
| `dream` | — | 2 | Library facade re-exporting all core crates (Layer 0, *, 1) |
| `assistant` | `clt` | 3 | Super-app aggregating all Layer 2 CLIs |

`*` `claude_storage_core` is a zero-dep parsing primitive sitting outside the layer hierarchy — no dependency on `claude_common`.

## Architecture

```
*        claude_storage_core      (zero-dep JSONL parser — no claude_common dep)
Layer 0: claude_common            (shared primitives — zero workspace deps)
             ↓
Layer 1: claude_profile_core      (token status, account domain logic)
         claude_version_core      (version, settings domain helpers)
         claude_runner_core       (ClaudeCommand builder + execute())
         claude_assets_core       (symlink artifact installer domain logic)
             ↓
Layer 2: dream       (lib)    (library facade — re-exports all core crates: Layer 0, *, 1)
         claude_profile  (clp)    (account management, token status)
         claude_storage  (clg)    (storage exploration)
         claude_runner   (clr)    (Claude Code execution)
         claude_version  (clv)  (install, version, session management)
         claude_assets   (cla)    (artifact installer: rules, skills, commands)
             ↓
Layer 3: assistant    (clt)    (super-app: all five Layer 2 CLIs)
```
