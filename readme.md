# assistant

Rust workspace for coding agent integration infrastructure. Currently targets Claude Code; architecture designed to extend to any coding agent.

## Quick Start

```bash
cargo install --path module/assistant

clv .status                  # version, token health, active processes
clv .version.install         # install or upgrade Claude Code

clp .account.list            # saved accounts (credential rotation)
clp .token.status            # active token — expiry and health

clr "review this file"       # run Claude Code with session continuity

clg .search "auth"           # search across session history

ast .help                    # all ~40 commands in one place
```

## Structure

| Path | Responsibility |
|------|----------------|
| `module/` | Fourteen workspace crates (see Crates below) |
| `docs/` | Workspace doc entities: feature, invariant, pattern, integration, Claude Code knowledge |
| `task/` | Task tracking: active, completed, backlog |
| `run/` | Container runner: scripts, Dockerfile, config, and variability analysis docs. |
| `verb/` | Universal Action Protocol: per-verb reference docs for all 7 `do` protocol verbs. |
| `vision.md` | Project vision, design rationale, and open problems |
| `../locales.md` | Locale and internationalisation notes |
| `Cargo.toml` | Workspace manifest: members, lints, shared dependencies |

## Crates

| Crate | Cmd | Layer | Responsibility |
|---|---|---|---|
| `claude_core` | — | 0 | Shared primitives: `ClaudePaths`, process utilities |
| `claude_storage_core` | — | * | Zero-dep JSONL parser for `~/.claude/`; path encoding |
| `claude_quota` | — | * | Anthropic API rate-limit HTTP transport; `RateLimitData`, `QuotaError` |
| `claude_profile_core` | — | 1 | Token status + account domain logic |
| `claude_version_core` | — | 1 | Version detection, install, settings domain helpers |
| `claude_runner_core` | — | 1 | `ClaudeCommand` builder + single process execution point |
| `claude_assets_core` | — | 1 | Symlink-based artifact installer domain logic |
| `claude_profile` | `clp` | 2 | Account management, token status, `~/.claude/` paths |
| `claude_storage` | `clg` | 2 | CLI for exploring Claude Code filesystem storage |
| `claude_runner` | `clr` | 2 | Claude Code execution with session continuity |
| `claude_version` | `clv` | 2 | Claude Code version manager |
| `claude_assets` | `cla` | 2 | Install Claude Code artifacts (rules, skills, commands) via symlinks |
| `dream` | — | 2 | Library facade re-exporting all core crates (Layer 0, *, 1) |
| `assistant` | `ast` | 3 | Super-app aggregating all Layer 2 CLIs |

`*` `claude_storage_core` is a zero-dep parsing primitive sitting outside the layer hierarchy — no dependency on `claude_core`.

## Architecture

```
*        claude_storage_core      (zero-dep JSONL parser — no claude_core dep)
*        claude_quota             (Anthropic API rate-limit HTTP transport — standalone primitive)
Layer 0: claude_core              (shared primitives — zero workspace deps)
             ↓
Layer 1: claude_profile_core      (token status, account domain logic)
         claude_version_core      (version, settings domain helpers)
         claude_runner_core       (ClaudeCommand builder + execute())
         claude_assets_core       (symlink artifact installer domain logic)
             ↓
Layer 2: dream           (lib)    (library facade — re-exports all core crates: Layer 0, *, 1)
         claude_profile  (clp)    (account management, token status)
         claude_storage  (clg)    (storage exploration)
         claude_runner   (clr)    (Claude Code execution)
         claude_version  (clv)    (Claude Code version manager)
         claude_assets   (cla)    (artifact installer: rules, skills, commands)
             ↓
Layer 3: assistant       (ast)    (super-app — all Layer 2 CLIs)
```

## Testing

**Container (all tests — real ~/.claude/ required):**
```bash
./run/runbox .test
```

**Container (offline — no ~/.claude/ needed):**
```bash
./run/runbox .test.offline
```

**Container (interactive shell):**
```bash
./run/runbox .shell
```

**Local (w3 required):**
```bash
./run/test
```
