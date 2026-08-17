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
| `module/` | Twenty-one workspace crates (see Crates below) |
| `contract/` | Behavioral contract test suites for external dependencies |
| `docs/` | Workspace doc entities: feature, invariant, pattern, integration, error |
| `../task/workspace/` | Workspace task registry — External Layout (see `../task/`) |
| `verb/` | Universal Action Protocol: workspace verbs + per-verb reference docs |
| `runbox/` | Owning container config for the globally-installed `runbox` engine |
| `vision.md` | Project vision, design rationale, and open problems |
| `Cargo.toml` | Workspace manifest: members, lints, shared dependencies |

## Crates

| Crate | Cmd | Layer | Responsibility |
|---|---|---|---|
| `claude_core` | — | 0 | Shared primitives: `ClaudePaths`, process utilities |
| `claude_storage_core` | — | * | Zero-dep JSONL parser for `~/.claude/`; path encoding |
| `claude_auth` | — | * | Anthropic OAuth token refresh transport; `TokenRefreshResult`, `AuthError` |
| `claude_quota` | — | * | Anthropic API rate-limit HTTP transport; `RateLimitData`, `QuotaError` |
| `claude_journal` | — | * | Append-only event journal library (zero workspace deps) |
| `json_redact` | — | * | Domain-agnostic redaction of sensitive values from strings and JSON |
| `svg_chart` | — | * | Minimal SVG line/bar chart rendering |
| `claude_profile_core` | — | 1 | Token status + account domain logic |
| `claude_version_core` | — | 1 | Version detection, install, settings domain helpers |
| `claude_runner_core` | — | 1 | `ClaudeCommand` builder + single process execution point |
| `claude_assets_core` | — | 1 | Symlink-based artifact installer domain logic |
| `claude_journal_charts` | — | 1 | Aggregates journal Command events into a daily-usage SVG bar chart |
| `claude_profile` | `clp` | 2 | Account management, token status, `~/.claude/` paths |
| `claude_storage` | `clg` | 2 | CLI for exploring Claude Code filesystem storage |
| `claude_runner` | `clr` | 2 | Claude Code execution with session continuity |
| `claude_version` | `clv` | 2 | Claude Code version manager |
| `claude_assets` | `cla` | 2 | Install Claude Code artifacts (rules, skills, commands) via symlinks |
| `claude_journal_viewer` | `clj` | 2 | Journal viewer CLI over `claude_journal` events |
| `dream` | — | 2 | Library facade re-exporting all core crates (Layer 0, *, 1) |
| `assistant` | `ast` | 3 | Super-app aggregating all Layer 2 CLIs |
| `assistant_kit` | — | 3 | Agent-agnostic integration layer library |

`*` Six crates (`claude_storage_core`, `claude_auth`, `claude_quota`, `claude_journal`, `json_redact`, `svg_chart`) sit outside the layer hierarchy — standalone primitives with no workspace dependencies.

## Architecture

```
*        claude_storage_core      (zero-dep JSONL parser — no claude_core dep)
*        claude_auth              (Anthropic OAuth token refresh transport — standalone primitive)
*        claude_quota             (Anthropic API rate-limit HTTP transport — standalone primitive)
*        claude_journal           (append-only event journal library — standalone primitive)
*        json_redact              (sensitive-value redaction — standalone primitive)
*        svg_chart                (SVG line/bar chart rendering — standalone primitive)
Layer 0: claude_core              (shared primitives — zero workspace deps)
             ↓
Layer 1: claude_profile_core      (token status, account domain logic)
         claude_version_core      (version, settings domain helpers)
         claude_runner_core       (ClaudeCommand builder + execute())
         claude_assets_core       (symlink artifact installer domain logic)
         claude_journal_charts    (journal events → daily-usage SVG chart)
             ↓
Layer 2: dream           (lib)    (library facade — re-exports all core crates: Layer 0, *, 1)
         claude_profile  (clp)    (account management, token status)
         claude_storage  (clg)    (storage exploration)
         claude_runner   (clr)    (Claude Code execution)
         claude_version  (clv)    (Claude Code version manager)
         claude_assets   (cla)    (artifact installer: rules, skills, commands)
         claude_journal_viewer (clj) (journal viewer over claude_journal events)
             ↓
Layer 3: assistant       (ast)    (super-app — all Layer 2 CLIs)
         assistant_kit   (lib)    (agent-agnostic integration layer)
```

## Testing

**Container (full workspace suite — real ~/.claude/ required):**
```bash
./verb/test
```

**Container (targeted nextest filter):**
```bash
./verb/test1 'test(name_substring)'
```

**Container (interactive shell):**
```bash
runbox .shell
```

**Host escape hatch (no container; honored by the nextest setup script):**
```bash
VERB_LAYER=l0 cargo nextest run --workspace --all-features
```
