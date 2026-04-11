> **Generated.** Do not edit manually. Maintained by `.locale.doc.generate`.
> Source of truth: `locales.config.yml` + `.persistent/locale.toml`.

# Locales — agent_kit

A **locale** is a named, bounded directory representing a self-contained unit of development work. See [`willbe/locate/module/locate/docs/locale.md`](../../willbe/locate/module/locate/docs/locale.md) for the full specification.

All paths are relative to `~/pro/lib/wip_core/claude_tools/dev`. `task` — Y = `task/` directory initialized.

## Summary

| # | rel-path | name | type | lang | purpose | task | last_active |
|---|----------|------|------|------|---------|------|-------------|
| 1 | `module/claude_common` | claude_common | rust_crate | rs | Shared primitives: ClaudePaths, process utilities | N | 2026-03-28 |
| 2 | `module/claude_storage_core` | claude_storage_core | rust_crate | rs | Zero-dep JSONL parser for ~/.claude/; path encoding | N | 2026-03-14 |
| 3 | `module/claude_profile_core` | claude_profile_core | rust_crate | rs | Token status and account domain logic | N | 2026-03-28 |
| 4 | `module/claude_manager_core` | claude_manager_core | rust_crate | rs | Version detection, install, settings domain helpers | N | 2026-03-29 |
| 5 | `module/claude_runner_core` | claude_runner_core | rust_crate | rs | ClaudeCommand builder and single process execution point | Y | 2026-04-04 |
| 6 | `module/claude_profile` | claude_profile | rust_crate | rs | Account credential management, token status, ~/.claude/ paths | N | 2026-04-11 |
| 7 | `module/claude_storage` | claude_storage | rust_crate | rs | CLI for exploring Claude Code filesystem storage | Y | 2026-04-11 |
| 8 | `module/claude_runner` | claude_runner | rust_crate | rs | Claude Code execution with session continuity | N | 2026-04-06 |
| 9 | `module/claude_manager` | claude_manager | rust_crate | rs | Install, version, session, and settings management | N | 2026-03-29 |
| 10 | `module/claude_tools` | claude_tools | rust_crate | rs | Super-app aggregating all Layer 2 CLIs | N | 2026-03-29 |
| 11 | `module/agent_kit` | agent_kit | rust_crate | rs | Library facade re-exporting all Layer 0–1 core crates | N | 2026-04-11 |

---

## Profile

### workspace :: agent_kit

| field | value |
|-------|-------|
| path | `lib/wip_core/claude_tools/dev` |
| parent | `lib/wip_core` |
| type | rust_workspace |
| lang | rs |
| canonical | Y |
| task | Y |
| last_active | 2026-04-11 |

**Purpose.** AI agent tooling infrastructure. Provides credential management, session state persistence (`claude_profile`, `claude_storage`, `claude_storage_core`) and execution management (`claude_runner_core`, `claude_runner`) for autonomous multi-turn workflows — currently focused on Claude Code, designed to expand to other agents.
